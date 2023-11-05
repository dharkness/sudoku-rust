use crate::layout::{Cell, Known};
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
        known: Known,
    ) -> ChangeResult {
        self.apply(board, &Action::new_set(strategy, cell, known))
    }

    /// Solves a single cell to one of its candidates.
    pub fn set_known(
        &self,
        board: &Board,
        strategy: Strategy,
        cell: Cell,
        known: Known,
    ) -> ChangeResult {
        self.apply(board, &Action::new_set(strategy, cell, known))
    }

    /// Removes a candidate from a single cell.
    pub fn remove_candidate(
        &self,
        board: &Board,
        strategy: Strategy,
        cell: Cell,
        known: Known,
    ) -> ChangeResult {
        self.apply(board, &Action::new_erase(strategy, cell, known))
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

            if self.options.solve_intersection_removals && next.is_empty() {
                if let Some(effects) = find_intersection_removals(&good) {
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
