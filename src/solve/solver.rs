use std::collections::HashMap;
use std::time::Instant;

use crate::io::{Cancelable, Parse, ParsePacked};
use crate::layout::{Cell, Known};
use crate::puzzle::{Action, Board, Effects};
use crate::solve::{find_brute_force, Difficulty, Reporter, MANUAL_TECHNIQUES};

pub enum Resolution {
    /// Returned when the givens for the initial puzzle are invalid
    /// along with the invalid board state and the cell that caused it.
    ///
    /// FIXME return the last valid board state instead of the invalid one
    Invalid(Board, Cell, Known, Effects),

    /// Returned when the puzzle is made invalid by one of the strategies
    /// along with the invalid board and errors the strategy caused.
    Failed(Board, Action, Effects),

    /// Returned when the puzzle cannot be solved using the available techniques.
    Unsolved(Board),

    /// Returned when the puzzle is completely solved along with the solution
    /// and the maximum difficulty of the strategies employed.
    Solved(Board, Difficulty),
}

impl Resolution {
    pub fn is_solved(&self) -> bool {
        matches!(self, Resolution::Solved(_, _))
    }
}

/// Attempts to solve puzzles using the available strategy algorithms.
pub struct Solver<'a> {
    /// Parses input to create each puzzle.
    parser: ParsePacked,

    /// Receives notifications about the result of solving each puzzle.
    ///
    /// The reporter receives the runtime and number of times each
    /// strategy was employed in addition to what's returned from [`solve()`].
    reporter: &'a dyn Reporter,

    /// The check option for the solve command verifies that the puzzle is solvable
    /// after each action to detect when an algorithm gives faulty deductions.
    check: bool,
}

impl Solver<'_> {
    pub fn new(reporter: &'_ dyn Reporter, check: bool) -> Solver<'_> {
        Solver {
            parser: Parse::packed().remove_peers().stop_on_error(),
            reporter,
            check,
        }
    }

    pub fn solve(&self, givens: &str, cancelable: &Cancelable) -> Resolution {
        let runtime = Instant::now();
        let (start, mut effects, failure) = self.parser.parse(givens);

        if let Some((cell, known)) = failure {
            self.reporter
                .invalid(givens, &start, &effects, cell, known, runtime.elapsed());
            return Resolution::Invalid(start, cell, known, effects);
        }

        let mut board = start;
        let mut counts = HashMap::new();
        let mut difficulty = Difficulty::Basic;

        loop {
            while effects.has_actions() {
                let mut next = Effects::new();
                for action in effects.actions() {
                    if cancelable.is_canceled() {
                        return Resolution::Unsolved(board);
                    }

                    let mut clone = board;
                    if action.apply(&mut clone, &mut next) {
                        let failed = if self.check && !clone.is_solved() {
                            find_brute_force(&clone, cancelable, false, 0).is_none()
                        } else {
                            next.has_errors()
                        };

                        if failed {
                            self.reporter.failed(
                                givens,
                                &start,
                                &board,
                                action,
                                &next,
                                runtime.elapsed(),
                                &counts,
                            );
                            return Resolution::Failed(board, action.clone(), next);
                        }

                        board = clone;
                        let count = counts.entry(action.strategy()).or_default();
                        *count += 1;
                    }
                }
                effects = next;
            }

            if board.is_solved() {
                self.reporter.solved(
                    givens,
                    &start,
                    &board,
                    difficulty,
                    runtime.elapsed(),
                    &counts,
                );
                return Resolution::Solved(board, difficulty);
            }

            let mut found = false;
            for solver in MANUAL_TECHNIQUES {
                if cancelable.is_canceled() {
                    return Resolution::Unsolved(board);
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
                self.reporter
                    .unsolved(givens, &start, &board, runtime.elapsed(), &counts);
                return Resolution::Unsolved(board);
            }
        }
    }
}
