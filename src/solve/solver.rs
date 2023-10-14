use std::collections::HashMap;
use std::time::Instant;

use crate::io::{Cancelable, Parse, ParsePacked};
use crate::layout::{Cell, Known};
use crate::puzzle::{Action, Board, Change, Effects, Options, Player};
use crate::solve::{find_brute_force, Difficulty, Reporter, NON_PEER_TECHNIQUES};

pub enum Resolution {
    /// Returned when the user interrupts the solver.
    Canceled,

    /// Returned when the givens for the initial puzzle are invalid
    /// along with the invalid board state and the cell that caused it.
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
    pub fn is_canceled(&self) -> bool {
        matches!(self, Resolution::Canceled)
    }

    pub fn is_solved(&self) -> bool {
        matches!(self, Resolution::Solved(_, _))
    }
}

/// Attempts to solve puzzles using the available strategy algorithms.
pub struct Solver<'a> {
    /// Applies actions to the board to solve the puzzle.
    player: Player,

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
        let player = Player::new(Options::errors_and_peers());
        Solver {
            player,
            parser: Parse::packed_with_player(player),
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
                        return Resolution::Canceled;
                    }

                    match self.player.apply(&board, action) {
                        Change::None => (),
                        Change::Valid(after, mut actions) => {
                            board = *after;
                            next.take_actions(&mut actions);
                            let count = counts.entry(action.strategy()).or_default();
                            *count += 1;
                        }
                        Change::Invalid(before, _, action, errors) => {
                            if action.to_string() == "J1 ⇨ 6" {
                                println!("\ncannot set J1 to 6\n");
                                errors.print_errors();
                            }
                            if self.check
                                && find_brute_force(&start, cancelable, false, 0, 2).is_solved()
                            {
                                eprintln!("error: solver caused errors in solvable puzzle");
                            }
                            self.reporter.failed(
                                givens,
                                &start,
                                &before,
                                &action,
                                &errors,
                                runtime.elapsed(),
                                &counts,
                            );
                            return Resolution::Failed(board, action.clone(), errors);
                        }
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
            for solver in NON_PEER_TECHNIQUES {
                if cancelable.is_canceled() {
                    return Resolution::Canceled;
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
