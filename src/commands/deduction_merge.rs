use std::collections::{BTreeMap, BTreeSet};

use crate::layout::{Cell, Digit, DigitSet};
use crate::puzzle::{Action, Effects};

pub fn take_actions_with_rules(target: &mut Vec<Action>, from: Effects) {
    for action in from.actions().iter() {
        merge_action(target, action.clone());
    }
}

fn merge_action(target: &mut Vec<Action>, incoming: Action) {
    if incoming.is_empty() {
        return;
    }

    let mut index = 0;
    while index < target.len() {
        if target[index].strategy() != incoming.strategy() {
            index += 1;
            continue;
        }

        match decide_same_strategy(&target[index], &incoming) {
            SameStrategyDecision::IgnoreIncoming => return,
            SameStrategyDecision::ReplaceExisting => {
                target.remove(index);
                continue;
            }
            SameStrategyDecision::MergeIntoExisting(merged) => {
                target[index] = merged;
                return;
            }
            SameStrategyDecision::KeepBoth => {
                index += 1;
            }
        }
    }

    if should_ignore_cross_strategy(target, &incoming) {
        return;
    }

    target.push(incoming);
}

fn should_ignore_cross_strategy(target: &[Action], incoming: &Action) -> bool {
    let incoming_view = effect_view(incoming);
    let incoming_clues = clue_set(incoming);

    for existing in target.iter() {
        if existing.strategy() == incoming.strategy() {
            continue;
        }
        let existing_view = effect_view(existing);
        if sets_conflict_view(&existing_view, &incoming_view) {
            continue;
        }
        if effects_equal_view(&existing_view, &incoming_view) {
            return true;
        }
        if effects_subset_view(&incoming_view, &existing_view) {
            let existing_clues = clue_set(existing);
            if incoming_clues.is_subset(&existing_clues) {
                return true;
            }
        }
    }

    false
}

enum SameStrategyDecision {
    IgnoreIncoming,
    ReplaceExisting,
    MergeIntoExisting(Action),
    KeepBoth,
}

fn decide_same_strategy(existing: &Action, incoming: &Action) -> SameStrategyDecision {
    let existing_view = effect_view(existing);
    let incoming_view = effect_view(incoming);

    if sets_conflict_view(&existing_view, &incoming_view) {
        return SameStrategyDecision::KeepBoth;
    }

    let existing_clues = clue_set(existing);
    let incoming_clues = clue_set(incoming);
    let existing_count = existing_clues.len();
    let incoming_count = incoming_clues.len();
    let incoming_clues_subset = incoming_clues.is_subset(&existing_clues);
    let existing_clues_subset = existing_clues.is_subset(&incoming_clues);
    let clues_equal = incoming_clues_subset && existing_clues_subset;

    if effects_equal_view(&existing_view, &incoming_view) {
        if incoming_clues_subset && !existing_clues_subset {
            return SameStrategyDecision::ReplaceExisting;
        }
        if existing_clues_subset && !incoming_clues_subset {
            return SameStrategyDecision::IgnoreIncoming;
        }
        if incoming_count < existing_count {
            return SameStrategyDecision::ReplaceExisting;
        }
        return SameStrategyDecision::IgnoreIncoming;
    }

    let incoming_subset = effects_subset_view(&incoming_view, &existing_view);
    let existing_subset = effects_subset_view(&existing_view, &incoming_view);

    if incoming_subset && incoming_clues_subset {
        return SameStrategyDecision::IgnoreIncoming;
    }

    if existing_subset && existing_clues_subset {
        return SameStrategyDecision::ReplaceExisting;
    }

    if !incoming_subset
        && !existing_subset
        && effects_overlap_view(&existing_view, &incoming_view)
        && clues_equal
    {
        let merged = merge_effects_keep_clues(existing, incoming);
        return SameStrategyDecision::MergeIntoExisting(merged);
    }

    SameStrategyDecision::KeepBoth
}

#[derive(Clone, Debug)]
struct EffectView {
    sets: BTreeMap<Cell, Digit>,
    erases: BTreeMap<Cell, DigitSet>,
}

fn effect_view(action: &Action) -> EffectView {
    let sets: BTreeMap<Cell, Digit> = action.collect_sets().collect();
    let mut erases = BTreeMap::new();
    for (cell, digits) in action.collect_erases() {
        if !sets.contains_key(&cell) {
            erases.insert(cell, digits);
        }
    }
    EffectView { sets, erases }
}

fn clue_set(action: &Action) -> BTreeSet<(Cell, Digit)> {
    action
        .collect_clues()
        .map(|(cell, digit, _)| (cell, digit))
        .collect()
}

fn sets_conflict_view(a: &EffectView, b: &EffectView) -> bool {
    a.sets
        .iter()
        .any(|(cell, digit)| b.sets.get(cell).is_some_and(|other| other != digit))
}

fn effects_equal_view(a: &EffectView, b: &EffectView) -> bool {
    a.sets == b.sets && a.erases == b.erases
}

fn effects_subset_view(sub: &EffectView, sup: &EffectView) -> bool {
    for (cell, digit) in sub.sets.iter() {
        match sup.sets.get(cell) {
            Some(other) if other == digit => (),
            _ => return false,
        }
    }

    for (cell, digits) in sub.erases.iter() {
        if sup.sets.contains_key(cell) {
            continue;
        }
        match sup.erases.get(cell) {
            Some(other) if other.has_all(*digits) => (),
            _ => return false,
        }
    }

    true
}

fn effects_overlap_view(a: &EffectView, b: &EffectView) -> bool {
    for (cell, digit) in a.sets.iter() {
        if b.sets.get(cell).is_some_and(|other| other == digit) {
            return true;
        }
        if b.erases.contains_key(cell) {
            return true;
        }
    }

    for (cell, digits) in a.erases.iter() {
        if b.sets.contains_key(cell) {
            return true;
        }
        if let Some(other) = b.erases.get(cell) {
            if digits.has_any(*other) {
                return true;
            }
        }
    }

    false
}

fn merge_effects_keep_clues(preferred: &Action, other: &Action) -> Action {
    let preferred_view = effect_view(preferred);
    let other_view = effect_view(other);

    let mut sets = preferred_view.sets.clone();
    for (cell, digit) in other_view.sets.iter() {
        sets.entry(*cell).or_insert(*digit);
    }

    let mut erases = BTreeMap::new();
    for (cell, digits) in preferred_view.erases.iter() {
        if !sets.contains_key(cell) {
            erases.insert(*cell, *digits);
        }
    }
    for (cell, digits) in other_view.erases.iter() {
        if !sets.contains_key(cell) {
            erases
                .entry(*cell)
                .and_modify(|current| *current |= *digits)
                .or_insert(*digits);
        }
    }

    let mut merged = Action::new(preferred.strategy());
    for (cell, digit) in sets {
        merged.set(cell, digit);
    }
    for (cell, digits) in erases {
        merged.erase_digits(cell, digits);
    }
    for (cell, digit, verdict) in preferred.collect_clues() {
        merged.clue_cell_for_digit(verdict, cell, digit);
    }

    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::Strategy;
    use crate::puzzle::Verdict;
    use crate::*;

    fn aggregate(actions: Vec<Action>) -> Vec<Action> {
        let mut target = Vec::new();
        for action in actions {
            merge_action(&mut target, action);
        }
        target
    }

    #[test]
    fn same_effects_prefers_fewer_clues() {
        let mut existing = Action::new_set(Strategy::NakedSingle, cell!(A1), digit!(1));
        existing.erase(cell!(B2), digit!(3));
        existing.clue_cell_for_digit(Verdict::Primary, cell!(A1), digit!(1));
        existing.clue_cell_for_digit(Verdict::Secondary, cell!(B2), digit!(3));

        let mut incoming = Action::new_set(Strategy::NakedSingle, cell!(A1), digit!(1));
        incoming.erase(cell!(B2), digit!(3));
        incoming.clue_cell_for_digit(Verdict::Secondary, cell!(A1), digit!(1));

        let result = aggregate(vec![existing, incoming]);

        assert_eq!(1, result.len());
        assert_eq!(1, clue_set(&result[0]).len());
    }

    #[test]
    fn same_effects_keeps_existing_when_more_clues() {
        let mut existing = Action::new_set(Strategy::NakedSingle, cell!(A1), digit!(1));
        existing.erase(cell!(B2), digit!(3));
        existing.clue_cell_for_digit(Verdict::Primary, cell!(A1), digit!(1));

        let mut incoming = Action::new_set(Strategy::NakedSingle, cell!(A1), digit!(1));
        incoming.erase(cell!(B2), digit!(3));
        incoming.clue_cell_for_digit(Verdict::Primary, cell!(A1), digit!(1));
        incoming.clue_cell_for_digit(Verdict::Secondary, cell!(B2), digit!(3));

        let result = aggregate(vec![existing, incoming]);

        assert_eq!(1, result.len());
        assert_eq!(1, clue_set(&result[0]).len());
    }

    #[test]
    fn subset_effects_and_clues_subsets_keep_superset() {
        let mut existing = Action::new(Strategy::Erase);
        existing.erase_digits(cell!(A1), digits![1 3]);
        existing.clue_cell_for_digit(Verdict::Primary, cell!(A1), digit!(1));
        existing.clue_cell_for_digit(Verdict::Secondary, cell!(A1), digit!(3));

        let mut incoming = Action::new(Strategy::Erase);
        incoming.erase(cell!(A1), digit!(1));
        incoming.clue_cell_for_digit(Verdict::Secondary, cell!(A1), digit!(1));

        let result = aggregate(vec![existing, incoming]);

        assert_eq!(1, result.len());
        assert!(result[0].erases(cell!(A1), digit!(1)));
        assert!(result[0].erases(cell!(A1), digit!(3)));
    }

    #[test]
    fn subset_effects_existing_subset_replaced_by_superset() {
        let mut existing = Action::new(Strategy::Erase);
        existing.erase(cell!(A1), digit!(1));
        existing.clue_cell_for_digit(Verdict::Primary, cell!(A1), digit!(1));

        let mut incoming = Action::new(Strategy::Erase);
        incoming.erase_digits(cell!(A1), digits![1 3]);
        incoming.clue_cell_for_digit(Verdict::Primary, cell!(A1), digit!(1));
        incoming.clue_cell_for_digit(Verdict::Secondary, cell!(A1), digit!(3));

        let result = aggregate(vec![existing, incoming]);

        assert_eq!(1, result.len());
        assert!(result[0].erases(cell!(A1), digit!(1)));
        assert!(result[0].erases(cell!(A1), digit!(3)));
    }

    #[test]
    fn subset_effects_without_clue_subset_keep_both() {
        let mut existing = Action::new(Strategy::Erase);
        existing.erase_digits(cell!(A1), digits![1 3]);
        existing.clue_cell_for_digit(Verdict::Primary, cell!(A1), digit!(1));

        let mut incoming = Action::new(Strategy::Erase);
        incoming.erase(cell!(A1), digit!(1));
        incoming.clue_cell_for_digit(Verdict::Secondary, cell!(B2), digit!(2));

        let result = aggregate(vec![existing, incoming]);

        assert_eq!(2, result.len());
    }

    #[test]
    fn overlap_equal_clues_merges_effects() {
        let mut existing = Action::new(Strategy::Erase);
        existing.erase_digits(cell!(A1), digits![1 2]);
        existing.clue_cell_for_digit(Verdict::Primary, cell!(A1), digit!(1));

        let mut incoming = Action::new(Strategy::Erase);
        incoming.erase_digits(cell!(A1), digits![2 3]);
        incoming.clue_cell_for_digit(Verdict::Secondary, cell!(A1), digit!(1));

        let result = aggregate(vec![existing, incoming]);

        assert_eq!(1, result.len());
        assert!(result[0].erases(cell!(A1), digit!(1)));
        assert!(result[0].erases(cell!(A1), digit!(2)));
        assert!(result[0].erases(cell!(A1), digit!(3)));
    }

    #[test]
    fn overlap_with_different_clues_keeps_both() {
        let mut existing = Action::new(Strategy::Erase);
        existing.erase_digits(cell!(A1), digits![1 2]);
        existing.clue_cell_for_digit(Verdict::Primary, cell!(A1), digit!(1));

        let mut incoming = Action::new(Strategy::Erase);
        incoming.erase_digits(cell!(A1), digits![2 3]);
        incoming.clue_cell_for_digit(Verdict::Secondary, cell!(B2), digit!(2));

        let result = aggregate(vec![existing, incoming]);

        assert_eq!(2, result.len());
    }

    #[test]
    fn set_overrides_erase_when_merging() {
        let mut existing = Action::new_set(Strategy::Place, cell!(A1), digit!(1));
        existing.clue_cell_for_digit(Verdict::Primary, cell!(A1), digit!(1));

        let mut incoming = Action::new(Strategy::Place);
        incoming.erase_digits(cell!(A1), digits![2 3]);
        incoming.clue_cell_for_digit(Verdict::Secondary, cell!(A1), digit!(1));

        let result = aggregate(vec![existing, incoming]);

        assert_eq!(1, result.len());
        assert!(result[0].sets(cell!(A1), digit!(1)));
        assert!(!result[0].erases(cell!(A1), digit!(2)));
        assert!(!result[0].erases(cell!(A1), digit!(3)));
    }

    #[test]
    fn conflicting_sets_keep_both() {
        let existing = Action::new_set(Strategy::Place, cell!(A1), digit!(1));
        let incoming = Action::new_set(Strategy::Place, cell!(A1), digit!(2));

        let result = aggregate(vec![existing, incoming]);

        assert_eq!(2, result.len());
    }

    #[test]
    fn cross_strategy_equal_effects_ignored() {
        let mut existing = Action::new(Strategy::NakedPair);
        existing.erase(cell!(A1), digit!(1));

        let mut incoming = Action::new(Strategy::AlignedPairExclusion);
        incoming.erase(cell!(A1), digit!(1));

        let result = aggregate(vec![existing, incoming]);

        assert_eq!(1, result.len());
        assert!(result[0].has_strategy(Strategy::NakedPair));
    }

    #[test]
    fn cross_strategy_subset_effects_and_clues_ignored() {
        let mut existing = Action::new(Strategy::HiddenSingle);
        existing.erase_digits(cell!(A1), digits![1 2]);
        existing.clue_cell_for_digit(Verdict::Primary, cell!(A1), digit!(1));
        existing.clue_cell_for_digit(Verdict::Secondary, cell!(A1), digit!(2));

        let mut incoming = Action::new(Strategy::XWing);
        incoming.erase(cell!(A1), digit!(1));
        incoming.clue_cell_for_digit(Verdict::Secondary, cell!(A1), digit!(1));

        let result = aggregate(vec![existing, incoming]);

        assert_eq!(1, result.len());
        assert!(result[0].has_strategy(Strategy::HiddenSingle));
    }

    #[test]
    fn cross_strategy_subset_effects_without_clue_subset_keeps_both() {
        let mut existing = Action::new(Strategy::HiddenSingle);
        existing.erase_digits(cell!(A1), digits![1 2]);
        existing.clue_cell_for_digit(Verdict::Primary, cell!(A1), digit!(1));

        let mut incoming = Action::new(Strategy::XWing);
        incoming.erase(cell!(A1), digit!(1));
        incoming.clue_cell_for_digit(Verdict::Secondary, cell!(B2), digit!(2));

        let result = aggregate(vec![existing, incoming]);

        assert_eq!(2, result.len());
    }
}
