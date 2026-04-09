use super::*;

pub fn find_almost_locked_sets(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();
    let als_sets = find_als_sets(board);

    for i in 0..als_sets.len() {
        for j in (i + 1)..als_sets.len() {
            let first = &als_sets[i];
            let second = &als_sets[j];

            if first.cells.has_any(second.cells) {
                continue;
            }

            let common = first.digits & second.digits;
            if common.is_empty() {
                continue;
            }

            let rcc = restricted_commons(first, second, common);
            if rcc.is_empty() {
                continue;
            }

            let z_digits = common - rcc;

            if rcc.len() == 1 {
                if add_rule_1(board, first, second, rcc, z_digits, single, &mut effects) {
                    return Some(effects);
                }
            } else {
                if add_rule_1_double_linked(board, first, second, rcc, single, &mut effects) {
                    return Some(effects);
                }
                if add_rule_2_double_linked(board, first, second, rcc, single, &mut effects) {
                    return Some(effects);
                }
            }
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

#[derive(Clone)]
struct Als {
    cells: CellSet,
    digits: DigitSet,
    digit_cells: [CellSet; 9],
}

fn find_als_sets(board: &Board) -> Vec<Als> {
    let mut als_sets = Vec::new();

    for house in House::iter() {
        let cells: Vec<Cell> = house
            .cells()
            .iter()
            .filter(|cell| board.candidates(*cell).len() >= 2)
            .collect();
        let candidates: Vec<DigitSet> = cells.iter().map(|cell| board.candidates(*cell)).collect();
        let cell_count = cells.len();
        if cell_count == 0 {
            continue;
        }

        let total_masks = 1usize << cell_count;
        for mask in 1..total_masks {
            let size = mask.count_ones() as usize;
            if size == 0 {
                continue;
            }

            let mut digits = DigitSet::empty();
            for (index, cell_digits) in candidates.iter().enumerate() {
                if (mask & (1usize << index)) != 0 {
                    digits |= *cell_digits;
                }
            }

            if digits.len() != size + 1 {
                continue;
            }

            let mut subset_cells = CellSet::empty();
            let mut digit_cells = [CellSet::empty(); 9];
            for (index, cell_digits) in candidates.iter().enumerate() {
                if (mask & (1usize << index)) != 0 {
                    let cell = cells[index];
                    subset_cells.add(cell);
                    for digit in *cell_digits {
                        digit_cells[digit.usize()].add(cell);
                    }
                }
            }

            als_sets.push(Als {
                cells: subset_cells,
                digits,
                digit_cells,
            });
        }
    }

    als_sets
}

fn restricted_commons(first: &Als, second: &Als, digits: DigitSet) -> DigitSet {
    let mut rcc = DigitSet::empty();
    for digit in digits {
        if is_restricted_common(first, second, digit) {
            rcc += digit;
        }
    }
    rcc
}

fn is_restricted_common(first: &Als, second: &Als, digit: Digit) -> bool {
    let first_cells = first.digit_cells[digit.usize()];
    let second_cells = second.digit_cells[digit.usize()];
    if first_cells.is_empty() || second_cells.is_empty() {
        return false;
    }

    for cell in first_cells {
        if !cell.peers().has_all(second_cells) {
            return false;
        }
    }
    for cell in second_cells {
        if !cell.peers().has_all(first_cells) {
            return false;
        }
    }

    true
}

fn add_rule_1(
    board: &Board,
    first: &Als,
    second: &Als,
    rcc: DigitSet,
    z_digits: DigitSet,
    single: bool,
    effects: &mut Effects,
) -> bool {
    if z_digits.is_empty() {
        return false;
    }

    for digit in z_digits {
        let required = first.digit_cells[digit.usize()] | second.digit_cells[digit.usize()];
        let mut erase_cells = CellSet::empty();
        for cell in board.candidate_cells(digit) {
            if first.cells.has(cell) || second.cells.has(cell) {
                continue;
            }
            if cell.peers().has_all(required) {
                erase_cells.add(cell);
            }
        }

        if erase_cells.is_empty() {
            continue;
        }

        let mut action = Action::new(Strategy::AlmostLockedSets);
        action.erase_cells(erase_cells, digit);
        add_als_clues(&mut action, first, Verdict::Primary, rcc + digit);
        add_als_clues(&mut action, second, Verdict::Secondary, rcc + digit);

        if effects.add_action(action) && single {
            return true;
        }
    }

    false
}

fn add_rule_1_double_linked(
    board: &Board,
    first: &Als,
    second: &Als,
    rcc: DigitSet,
    single: bool,
    effects: &mut Effects,
) -> bool {
    for digit in rcc {
        let required = first.digit_cells[digit.usize()] | second.digit_cells[digit.usize()];
        let mut erase_cells = CellSet::empty();
        for cell in board.candidate_cells(digit) {
            if first.cells.has(cell) || second.cells.has(cell) {
                continue;
            }
            if cell.peers().has_all(required) {
                erase_cells.add(cell);
            }
        }

        if erase_cells.is_empty() {
            continue;
        }

        let mut action = Action::new(Strategy::AlmostLockedSets);
        action.erase_cells(erase_cells, digit);
        add_als_clues(&mut action, first, Verdict::Primary, DigitSet::of(digit));
        add_als_clues(&mut action, second, Verdict::Secondary, DigitSet::of(digit));

        if effects.add_action(action) && single {
            return true;
        }
    }

    false
}

fn add_rule_2_double_linked(
    board: &Board,
    first: &Als,
    second: &Als,
    rcc: DigitSet,
    single: bool,
    effects: &mut Effects,
) -> bool {
    let first_unique = first.digits - second.digits;
    let second_unique = second.digits - first.digits;

    if add_unique_digit_elims(board, first, first_unique, rcc, single, effects) {
        return true;
    }
    if add_unique_digit_elims(board, second, second_unique, rcc, single, effects) {
        return true;
    }

    false
}

fn add_unique_digit_elims(
    board: &Board,
    als: &Als,
    unique: DigitSet,
    rcc: DigitSet,
    single: bool,
    effects: &mut Effects,
) -> bool {
    if unique.is_empty() {
        return false;
    }

    for digit in unique {
        let mut erase_cells = CellSet::empty();
        for cell in board.candidate_cells(digit) {
            if als.cells.has(cell) {
                continue;
            }
            if cell.peers().has_all(als.cells) {
                erase_cells.add(cell);
            }
        }

        if erase_cells.is_empty() {
            continue;
        }

        let mut action = Action::new(Strategy::AlmostLockedSets);
        action.erase_cells(erase_cells, digit);
        add_als_clues(
            &mut action,
            als,
            Verdict::Primary,
            rcc | DigitSet::of(digit),
        );

        if effects.add_action(action) && single {
            return true;
        }
    }

    false
}

fn add_als_clues(action: &mut Action, als: &Als, verdict: Verdict, digits: DigitSet) {
    for digit in digits {
        action.clue_cells_for_digit(verdict, als.digit_cells[digit.usize()], digit);
    }
}

#[cfg(test)]
mod tests {
    use crate::io::{Parse, Parser};
    use crate::puzzle::Options;
    use crate::*;

    use super::*;

    #[test]
    fn rule_1_example_1() {
        let parser = Parse::packed_with_options(Options::all());
        let (board, effects, failed) = parser.parse(
            "910700003062003709735004086009372060023050070057048932270400300501237690390000027",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_almost_locked_sets(&board, false).expect("not found");
        assert_erase!(got, A7, 4);
    }

    #[test]
    fn rule_1_example_2() {
        let parser = Parse::packed_with_options(Options::all());
        let (board, effects, failed) = parser.parse(
            "030680105601503000000001060004805630860000051057000980070408000000700008408050020",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_almost_locked_sets(&board, false).expect("not found");
        assert_erase!(got, B2, 2);
    }

    #[test]
    fn rule_1_example_3() {
        let parser = Parse::packed_with_options(Options::all());
        let (board, effects, failed) = parser.parse(
            "006805030034600850580000010893100400000090003000000901300000040075003160648201390",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_almost_locked_sets(&board, false).expect("not found");
        assert_erase!(got, E2, 1);
    }

    #[test]
    fn rule_1_example_4() {
        let parser = Parse::packed_with_options(Options::all());
        let (board, effects, failed) = parser.parse(
            "006805030034600850580000010893100400000090003000000901300000040075003160648201390",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_almost_locked_sets(&board, false).expect("not found");
        assert_erase!(got, E2, 1);
    }

    #[test]
    fn double_linked_example() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9Ba49xa358d8bgay03aq069g9m059s8001089ea208031m017uay023m4d0l0r090507031m4z6n062r0r4e0v6a0902bz9za344541r6a2i6b2e2e0643b6b7020504820408071g1w8i0103838302031m22du36c2",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_almost_locked_sets(&board, false).expect("not found");
        assert_erase!(got, A3, 4);
    }
}
