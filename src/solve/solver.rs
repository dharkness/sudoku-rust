use crate::io::Cancelable;
use crate::puzzle::{Action, Board, ChangeResult, Changer, Difficulty, Effects, Options};
use crate::solve::{find_brute_force, NON_PEER_TECHNIQUES};

pub enum Resolution {
    /// Returned when the user interrupts the solver
    /// along with the current puzzle state and actions applied.
    Canceled(Board, Effects, Difficulty),

    /// Returned when the puzzle is made invalid by one of the strategies
    /// along with the invalid board, the valid actions applied,
    /// and the action and errors the strategy caused.
    Failed(Board, Effects, Difficulty, Action, Effects),

    /// Returned when the puzzle cannot be solved using the available techniques
    /// along with the partially completed puzzle and the valid actions applied.
    Unsolved(Board, Effects, Difficulty),

    /// Returned when the puzzle is completely solved along with the solution,
    /// actions applied to find it, and the highest solver difficulty required.
    Solved(Board, Effects, Difficulty),
}

impl Resolution {
    pub fn is_canceled(&self) -> bool {
        matches!(self, Resolution::Canceled(..))
    }

    pub fn is_solved(&self) -> bool {
        matches!(self, Resolution::Solved(..))
    }
}

/// Attempts to solve puzzles using the available strategy algorithms.
pub struct Solver {
    /// Applies actions to the board to solve the puzzle.
    changer: Changer,

    /// Allows canceling the solver.
    cancelable: Cancelable,

    /// The check option for the solve command verifies that the puzzle is solvable
    /// after each action to detect when an algorithm gives faulty deductions.
    check: bool,
}

impl Solver {
    pub fn new(check: bool) -> Solver {
        Solver {
            changer: Changer::new(Options::errors()),
            cancelable: Cancelable::new(),
            check,
        }
    }

    pub fn solve(&self, start: &Board, unapplied: &Effects) -> Resolution {
        let mut board = *start;
        let mut effects = unapplied.clone();
        let mut applied = Effects::new();
        let mut difficulty = Difficulty::Basic;

        loop {
            while effects.has_actions() {
                let mut next = Effects::new();
                for action in effects.actions() {
                    if self.cancelable.is_canceled() {
                        return Resolution::Canceled(board, applied, difficulty);
                    }

                    match self.changer.apply(&board, action) {
                        ChangeResult::None => (),
                        ChangeResult::Valid(after, actions) => {
                            applied.add_action(action.clone());
                            board = *after;
                            next.take_actions(actions);
                        }
                        ChangeResult::Invalid(before, _, action, errors) => {
                            if self.check && find_brute_force(start, false, 0, 2).is_solved() {
                                eprintln!(
                                    "error: solver caused errors in solvable puzzle: {}",
                                    start.packed_string()
                                );
                            }
                            return Resolution::Failed(
                                *before,
                                applied,
                                difficulty,
                                action.clone(),
                                errors,
                            );
                        }
                    }
                }
                effects = next;
            }

            if board.is_fully_solved() {
                return Resolution::Solved(board, applied, difficulty);
            }

            let mut found = false;
            for solver in NON_PEER_TECHNIQUES {
                if self.cancelable.is_canceled() {
                    return Resolution::Canceled(board, applied, difficulty);
                }

                if let Some(moves) = solver.solve(&board) {
                    if solver.difficulty() > difficulty {
                        difficulty = solver.difficulty()
                    }
                    effects = moves;
                    found = true;
                    break;
                }
            }

            if !found {
                return Resolution::Unsolved(board, applied, difficulty);
            }
        }
    }
}
