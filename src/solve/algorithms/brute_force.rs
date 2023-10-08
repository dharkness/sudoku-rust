use std::thread::sleep;
use std::time::Duration;

use super::*;
use crate::io::{print_candidates, Cancelable};

const MINIMUM_KNOWNS_TO_BE_UNIQUELY_SOLVABLE: usize = 17;

const MAXIMUM_SOLUTIONS: usize = 1_000_000;
const DEFAULT_MAXIMUM_SOLUTIONS: usize = 1_000;

pub fn find_brute_force(
    board: &Board,
    cancelable: &Cancelable,
    log: bool,
    pause: u32,
    mut max_solutions: usize,
) -> BruteForceResult {
    if board.is_solved() {
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

    let player = Player::new(Options::all());
    let mut solutions: Vec<Effects> = Vec::new();
    let mut stack = Vec::with_capacity(81);
    stack.push(Entry::new(*board, Effects::new()));

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
            actions,
        } = stack.last_mut().unwrap();

        if candidates.is_empty() {
            if log {
                println!("backtrack\n");
            }
            stack.pop();
            continue;
        }

        if log {
            print_candidates(board);
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

        match player.apply(board, &action) {
            Change::None => (),
            Change::Valid(after, _) => {
                if log {
                    print_candidates(&after);
                }

                let mut actions = actions.clone();
                actions.add_action(action);

                if after.is_solved() {
                    solutions.push(actions);
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

                stack.push(Entry::new(*after, actions));
            }
            Change::Invalid(_, _, _, errors) => {
                if log {
                    println!("failed\n");
                    errors.print_errors();
                }
            }
        }
    }

    match solutions.len() {
        0 => BruteForceResult::Unsolvable,
        1 => BruteForceResult::Solved(solutions.pop().unwrap()),
        _ => BruteForceResult::MultipleSolutions(solutions),
    }
}

pub enum BruteForceResult {
    AlreadySolved,
    TooFewKnowns,
    UnsolvableCells(CellSet),
    Canceled,
    Unsolvable,
    Solved(Effects),
    MultipleSolutions(Vec<Effects>),
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
    actions: Effects,
}

impl Entry {
    pub fn new(board: Board, actions: Effects) -> Self {
        let cell = board.unknowns().first().unwrap();
        let candidates = board.candidates(cell);

        Self {
            board,
            cell,
            candidates,
            actions,
        }
    }
}
