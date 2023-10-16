use crate::io::Cancelable;
use crate::puzzle::{Action, Board, Change, Effects, Options, Player};
use crate::solve::{find_brute_force, Difficulty, NON_PEER_TECHNIQUES};

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
pub struct Solver<'a> {
    /// Applies actions to the board to solve the puzzle.
    player: Player,

    /// Allows canceling the solver.
    cancelable: &'a Cancelable,

    /// The check option for the solve command verifies that the puzzle is solvable
    /// after each action to detect when an algorithm gives faulty deductions.
    check: bool,
}

impl Solver<'_> {
    pub fn new(cancelable: &'_ Cancelable, check: bool) -> Solver<'_> {
        Solver {
            player: Player::new(Options::errors_and_peers()),
            cancelable,
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

                    match self.player.apply(&board, action) {
                        Change::None => (),
                        Change::Valid(after, actions) => {
                            applied.add_action(action.clone());
                            board = *after;
                            next.take_actions(actions);
                        }
                        Change::Invalid(before, _, action, errors) => {
                            if self.check
                                && find_brute_force(start, self.cancelable, false, 0, 2).is_solved()
                            {
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
