use std::thread::sleep;
use std::time::Duration;

use crate::io::{print_all_and_single_candidates, Cancelable};

use super::*;

const MINIMUM_KNOWNS_TO_BE_UNIQUELY_SOLVABLE: usize = 17;

const MAXIMUM_SOLUTIONS: usize = 1_000_000;
const DEFAULT_MAXIMUM_SOLUTIONS: usize = 1_000;

pub fn find_brute_force(
    board: &Board,
    log: bool,
    pause: u32,
    mut max_solutions: usize,
) -> BruteForceResult {
    if board.is_fully_solved() {
        return BruteForceResult::AlreadySolved;
    }
    if board.known_count() < MINIMUM_KNOWNS_TO_BE_UNIQUELY_SOLVABLE {
        return BruteForceResult::TooFewKnowns;
    }

    let empty = board.unknowns() & board.cells_with_n_candidates(0);
    if !empty.is_empty() {
        return BruteForceResult::UnsolvableCells(empty);
    }

    if !(1..=MAXIMUM_SOLUTIONS).contains(&max_solutions) {
        max_solutions = DEFAULT_MAXIMUM_SOLUTIONS;
    }

    let cancelable = Cancelable::new();
    let changer = Changer::new(Options::errors_and_peers());
    let mut solutions = Vec::new();
    let mut stack = Vec::with_capacity(81);
    stack.push(Entry::new(*board));

    while !stack.is_empty() {
        if cancelable.is_canceled() {
            return BruteForceResult::Canceled;
        }
        if log {
            println!("stack size {}\n", stack.len());
        }

        let Entry {
            board,
            cell,
            candidates,
        } = stack.last_mut().unwrap();

        if candidates.is_empty() {
            if log {
                println!("backtrack\n");
            }
            stack.pop();
            continue;
        }

        if log {
            print_all_and_single_candidates(board);
            println!("\ncell {} candidates {}\n", cell, candidates);
        };

        let known = candidates.pop().unwrap();
        let action = Action::new_set(Strategy::BruteForce, *cell, known);
        if log {
            println!("try {}\n", action);
            if pause > 0 {
                sleep(Duration::from_millis(1000));
            }
        }

        match changer.apply(board, &action) {
            ChangeResult::None => (),
            ChangeResult::Valid(after, _) => {
                if log {
                    print_all_and_single_candidates(&after);
                }

                if after.is_fully_solved() {
                    solutions.push(*after);
                    if log {
                        println!("found solution {}\n", solutions.len());
                    }
                    if solutions.len() >= max_solutions {
                        return BruteForceResult::MultipleSolutions(solutions);
                    } else {
                        if log {
                            println!("backtrack\n");
                        }
                        stack.pop();
                        continue;
                    }
                }

                stack.push(Entry::new(*after));
            }
            ChangeResult::Invalid(_, _, _, errors) => {
                if log {
                    println!("failed\n");
                    errors.print_errors();
                }
            }
        }
    }

    match solutions.len() {
        0 => BruteForceResult::Unsolvable,
        1 => BruteForceResult::Solved(Box::new(solutions[0])),
        _ => BruteForceResult::MultipleSolutions(solutions),
    }
}

pub enum BruteForceResult {
    AlreadySolved,
    TooFewKnowns,
    UnsolvableCells(CellSet),
    Canceled,
    Unsolvable,
    Solved(Box<Board>),
    MultipleSolutions(Vec<Board>),
}

impl BruteForceResult {
    pub fn is_solved(&self) -> bool {
        matches!(self, Self::AlreadySolved) || matches!(self, Self::Solved(_))
    }
}

struct Entry {
    board: Board,
    cell: Cell,
    candidates: KnownSet,
}

impl Entry {
    pub fn new(board: Board) -> Self {
        let cell = board.unknowns().first().unwrap();
        let candidates = board.candidates(cell);

        Self {
            board,
            cell,
            candidates,
        }
    }
}
