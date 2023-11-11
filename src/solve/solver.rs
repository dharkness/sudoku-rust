use std::time::Instant;

use crate::io::Cancelable;
use crate::puzzle::{Action, Board, ChangeResult, Changer, Difficulty, Effects, Options};
use crate::solve::{find_brute_force, Timings, NON_PEER_TECHNIQUES};

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

    pub fn solve(&self, start: &Board, _: &Effects, timings: &mut Timings) -> Resolution {
        let mut board = *start;
        let mut applied = Effects::new();
        let mut difficulty = Difficulty::Basic;

        loop {
            if board.is_fully_solved() {
                return Resolution::Solved(board, applied, difficulty);
            }

            let mut action = None;
            for solver in NON_PEER_TECHNIQUES {
                if self.cancelable.is_canceled() {
                    return Resolution::Canceled(board, applied, difficulty);
                }

                let runtime = Instant::now();
                if let Some(moves) = solver.solve(&board) {
                    timings.add(solver.strategy(), moves.action_count(), runtime.elapsed());
                    if solver.difficulty() > difficulty {
                        difficulty = solver.difficulty()
                    }
                    action = Some(moves.actions()[0].clone());
                    break;
                } else {
                    timings.add(solver.strategy(), 0, runtime.elapsed());
                }
            }

            if let Some(action) = action {
                match self.changer.apply(&board, &action) {
                    ChangeResult::None => (),
                    ChangeResult::Valid(after, _) => {
                        applied.add_action(action);
                        board = *after;
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
            } else {
                return Resolution::Unsolved(board, applied, difficulty);
            }
        }
    }
}
