use crate::layout::{Cell, Digit};
use crate::puzzle::{Change, Strategy};
use crate::solve::find_intersection_removals;

use super::{Action, Board, Effects, Options};

/// Indicates the result of a single manual action or any applied automatic actions.
pub enum ChangeResult {
    None,
    Valid(Box<Board>, Effects),
    Invalid(Box<Board>, Box<Board>, Action, Effects),
}

/// Applies manual and automatic actions to a board based on the selected options.
///
/// None of the methods modify the given board.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Changer {
    pub options: Options,
}

impl Changer {
    pub const fn new(options: Options) -> Self {
        Self { options }
    }

    /// Sets the given (clue) for a single cell.
    pub fn set_given(
        &self,
        board: &Board,
        strategy: Strategy,
        cell: Cell,
        digit: Digit,
    ) -> ChangeResult {
        self.apply(board, &Action::new_set(strategy, cell, digit))
    }

    /// Solves a single cell to one of its candidates.
    pub fn set_digit(
        &self,
        board: &Board,
        strategy: Strategy,
        cell: Cell,
        digit: Digit,
    ) -> ChangeResult {
        self.apply(board, &Action::new_set(strategy, cell, digit))
    }

    /// Removes a candidate from a single cell.
    pub fn remove_candidate(
        &self,
        board: &Board,
        strategy: Strategy,
        cell: Cell,
        digit: Digit,
    ) -> ChangeResult {
        self.apply(board, &Action::new_erase(strategy, cell, digit))
    }

    /// Applies the given action and any automatic actions it creates.
    pub fn apply(&self, board: &Board, action: &Action) -> ChangeResult {
        let mut after = *board;
        let mut effects = Effects::new();

        let change = action.apply(&mut after, &mut effects);
        if self.options.stop_on_error && effects.has_errors() {
            ChangeResult::Invalid(Box::new(*board), Box::new(after), action.clone(), effects)
        } else {
            self.apply_all_changed(board, &after, &effects, change)
        }
    }

    /// Applies all automatic actions to the given board.
    pub fn apply_all(&self, board: &Board, actions: &Effects) -> ChangeResult {
        self.apply_all_changed(board, board, actions, Change::None)
    }

    fn apply_all_changed(
        &self,
        before: &Board,
        board: &Board,
        actions: &Effects,
        mut change: Change,
    ) -> ChangeResult {
        let mut good = *board;
        let mut applying = actions.clone();
        let mut unapplied = Effects::new();

        while applying.has_actions() {
            let mut next = Effects::new();
            for action in applying.actions() {
                if self.options.should_apply(action.strategy()) {
                    let mut maybe = good;
                    change &= action.apply(&mut maybe, &mut next);
                    if self.options.stop_on_error && next.has_errors() {
                        return ChangeResult::Invalid(
                            Box::new(*before),
                            Box::new(maybe),
                            action.clone(),
                            next,
                        );
                    }
                    if next.has_errors() {
                        eprintln!("warning: action caused errors: {}", action);
                        next.print_errors();
                    }
                    good = maybe;
                } else {
                    unapplied.add_action(action.clone());
                }
            }

            // FIXME why only intersection removals and not naked/hidden singles?
            if self.options.solve_intersection_removals && next.is_empty() {
                if let Some(effects) = find_intersection_removals(&good, false) {
                    next = effects;
                }
            }

            applying = next;
        }

        if change.changed() {
            // errors are treated as valid when not stopping for them
            ChangeResult::Valid(Box::new(good), unapplied)
        } else {
            ChangeResult::None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::io::{Parse, Parser};
    use crate::layout::DigitSet;
    use crate::*;

    use super::*;

    fn board_with_single_candidate(cell: Cell, keep: Digit) -> Board {
        let mut board = Board::new();
        let mut effects = Effects::new();
        for digit in Digit::iter() {
            if digit != keep {
                board.remove_candidate(cell, digit, &mut effects);
            }
        }
        board
    }

    #[test]
    fn set_digit_returns_valid() {
        let board = Board::new();
        let changer = Changer::new(Options::none());

        let result = changer.set_digit(&board, Strategy::Place, cell!(A1), digit!(1));

        match result {
            ChangeResult::Valid(after, unapplied) => {
                assert_eq!(Some(digit!(1)), after.value(cell!(A1)).digit());
                assert!(unapplied.is_empty());
            }
            _ => panic!("expected valid change"),
        }
    }

    #[test]
    fn set_digit_same_value_returns_none() {
        let mut board = Board::new();
        let mut effects = Effects::new();
        board.set_given(cell!(A1), digit!(1), &mut effects);

        let changer = Changer::new(Options::none());
        let result = changer.set_digit(&board, Strategy::Place, cell!(A1), digit!(1));

        match result {
            ChangeResult::None => {}
            _ => panic!("expected no change"),
        }
    }

    #[test]
    fn stop_on_error_returns_invalid() {
        let board = board_with_single_candidate(cell!(A1), digit!(9));
        let changer = Changer::new(Options::errors());

        let result = changer.remove_candidate(&board, Strategy::Erase, cell!(A1), digit!(9));

        match result {
            ChangeResult::Invalid(before, after, action, effects) => {
                assert_eq!(digits![9], before.candidates(cell!(A1)));
                assert!(after.candidates(cell!(A1)).is_empty());
                assert!(effects.has_errors());
                assert!(action.has_strategy(Strategy::Erase));
            }
            _ => panic!("expected invalid change"),
        }
    }

    #[test]
    fn ignore_errors_returns_valid() {
        let board = board_with_single_candidate(cell!(A1), digit!(9));
        let changer = Changer::new(Options::none());

        let result = changer.remove_candidate(&board, Strategy::Erase, cell!(A1), digit!(9));

        match result {
            ChangeResult::Valid(after, _) => {
                assert!(after.candidates(cell!(A1)).is_empty());
            }
            _ => panic!("expected valid change"),
        }
    }

    #[test]
    fn apply_all_with_unapplied_actions_returns_none() {
        let board = Board::new();
        let changer = Changer::new(Options::none());

        let action = Action::new_erase(Strategy::NakedPair, cell!(A1), digit!(1));
        let mut effects = Effects::new();
        effects.add_action(action);

        let result = changer.apply_all(&board, &effects);

        match result {
            ChangeResult::None => {}
            _ => panic!("expected no change"),
        }
    }

    #[test]
    fn apply_all_runs_intersection_removals() {
        let board = Parse::packed_with_options(Options::errors()).parse_simple(
            "
                7..1....9
                .2.3..7..
                4.9......
                .6.8..2..
                .........
                .7...1.5.
                .....49..
                .46..5..2
                .1...68..
            ",
        );

        assert!(board.is_candidate(cell!(B8), digit!(1)));

        let changer = Changer::new(Options::none().solve_intersection_removals());
        let action = Action::new_erase(Strategy::NakedPair, cell!(A1), digit!(1));
        let mut effects = Effects::new();
        effects.add_action(action);

        let result = changer.apply_all(&board, &effects);

        match result {
            ChangeResult::Valid(after, unapplied) => {
                assert!(!after.is_candidate(cell!(B8), digit!(1)));
                assert!(unapplied.action_count() >= 1);
                assert!(unapplied
                    .actions()
                    .iter()
                    .any(|action| action.has_strategy(Strategy::NakedPair)));
            }
            _ => panic!("expected valid change"),
        }
    }
}
