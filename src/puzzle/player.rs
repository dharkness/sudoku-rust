use crate::layout::{Cell, Known};
use crate::puzzle::Strategy;
use crate::solve::find_intersection_removals;

use super::{Action, Board, Effects, Options};

/// Indicates the result of a single manual action or any applied automatic actions.
pub enum Change {
    None,
    Valid(Box<Board>, Effects),
    Invalid(Box<Board>, Box<Board>, Action, Effects),
}

/// Applies manual and automatic actions to a board based on the selected options.
///
/// None of the methods modify the given board.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Player {
    pub options: Options,
}

impl Player {
    pub const fn new(options: Options) -> Self {
        Self { options }
    }

    /// Sets the given (clue) for a single cell.
    pub fn set_given(&self, board: &Board, strategy: Strategy, cell: Cell, known: Known) -> Change {
        self.apply(board, &Action::new_set(strategy, cell, known))
    }

    /// Solves a single cell to one of its candidates.
    pub fn set_known(&self, board: &Board, strategy: Strategy, cell: Cell, known: Known) -> Change {
        self.apply(board, &Action::new_set(strategy, cell, known))
    }

    /// Removes a candidate from a single cell.
    pub fn remove_candidate(
        &self,
        board: &Board,
        strategy: Strategy,
        cell: Cell,
        known: Known,
    ) -> Change {
        self.apply(board, &Action::new_erase(strategy, cell, known))
    }

    /// Applies the given action and any automatic actions it creates.
    pub fn apply(&self, board: &Board, action: &Action) -> Change {
        let mut after = *board;
        let mut effects = Effects::new();

        if !action.apply(&mut after, &mut effects) {
            Change::None
        } else if self.options.stop_on_error && effects.has_errors() {
            Change::Invalid(Box::new(*board), Box::new(after), action.clone(), effects)
        } else {
            self.apply_all_changed(board, &after, &effects, true)
        }
    }

    /// Applies all automatic actions to the given board.
    pub fn apply_all(&self, board: &Board, actions: &Effects) -> Change {
        self.apply_all_changed(board, board, actions, false)
    }

    fn apply_all_changed(
        &self,
        before: &Board,
        board: &Board,
        actions: &Effects,
        mut changed: bool,
    ) -> Change {
        let mut good = *board;
        let mut applying = actions.clone();
        let mut unapplied = Effects::new();

        while applying.has_actions() {
            let mut next = Effects::new();
            for action in applying.actions() {
                if self.options.should_apply(action.strategy()) {
                    let mut maybe = good;
                    changed = action.apply(&mut maybe, &mut next) || changed;
                    if self.options.stop_on_error && next.has_errors() {
                        return Change::Invalid(
                            Box::new(*before),
                            Box::new(maybe),
                            action.clone(),
                            next,
                        );
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

        if changed {
            Change::Valid(Box::new(good), unapplied)
        } else {
            Change::None
        }
    }

    pub fn without(&self, board: &Board, cell: Cell) -> (Board, Effects) {
        match board.known_count() {
            0 => (*board, Effects::new()),
            1 => (Board::new(), Effects::new()),
            _ => board.known_iter().filter(|(c, _)| *c != cell).fold(
                (Board::new(), Effects::new()),
                |(mut b, mut e), (c, k)| {
                    b.set_given(c, k, &mut e);
                    (b, e)
                },
            ),
        }
    }
}
