use std::collections::HashMap;
use std::time::Instant;

use crate::io::{print_candidates, Parse, ParsePacked};
use crate::puzzle::Effects;
use crate::solve::{find_brute_force, Difficulty, Reporter, MANUAL_TECHNIQUES};

pub struct Solver<'a> {
    parser: ParsePacked,
    reporter: &'a dyn Reporter,
    check: bool,
}

impl Solver<'_> {
    pub fn new(reporter: &'_ dyn Reporter, check: bool) -> Solver<'_> {
        Solver {
            parser: Parse::packed().stop_on_error(),
            reporter,
            check,
        }
    }

    pub fn solve(&self, givens: &str) -> bool {
        let runtime = Instant::now();
        let (start, mut effects, failure) = self.parser.parse(givens);

        if let Some((cell, known)) = failure {
            self.reporter
                .invalid(givens, &start, &effects, cell, known, runtime.elapsed());
            return false;
        }

        let mut board = start;
        let mut counts = HashMap::new();
        let mut difficulty = Difficulty::Basic;

        loop {
            while effects.has_actions() {
                let mut next = Effects::new();
                for action in effects.actions() {
                    let mut clone = board;
                    if action.apply(&mut clone, &mut next) {
                        if self.check {
                            if find_brute_force(&clone).is_none() {
                                print_candidates(&board);
                                println!(
                                    "\nbrute force failed for {:?} {}",
                                    action.strategy(),
                                    action
                                );
                                return false;
                            }
                        }

                        if next.has_errors() {
                            self.reporter.failed(
                                givens,
                                &start,
                                &board,
                                action,
                                &next,
                                runtime.elapsed(),
                                &counts,
                            );
                            return false;
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
                return true;
            }

            let mut found = false;
            for solver in MANUAL_TECHNIQUES {
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
                return false;
            }
        }
    }
}
