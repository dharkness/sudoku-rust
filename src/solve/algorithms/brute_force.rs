use super::*;
use crate::io::{print_candidates, Cancelable};
use std::thread::sleep;
use std::time::Duration;

const MINIMUM_KNOWNS_TO_BE_SOLVABLE: usize = 17;

pub fn find_brute_force(
    board: &Board,
    cancelable: &Cancelable,
    log: bool,
    pause: u32,
) -> Option<Effects> {
    if board.is_solved() || board.known_count() < MINIMUM_KNOWNS_TO_BE_SOLVABLE {
        return None;
    }

    let mut stack = Vec::with_capacity(81);
    stack.push(Entry::new(*board, Effects::new()));

    while !stack.is_empty() {
        if cancelable.is_canceled() {
            return None;
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
        let mut clone = *board;
        let mut effects = Effects::new();

        if log {
            println!("try {}\n", known);
            if pause > 0 {
                sleep(Duration::from_millis(1000));
            }
        }

        clone.set_known(*cell, known, &mut effects);
        if log {
            print_candidates(&clone);
        }
        if effects.has_errors() {
            if log {
                println!("failed\n");
                effects.print_errors();
            }
        } else {
            if let Some(errors) = effects.apply_all(&mut clone) {
                if log {
                    print_candidates(&clone);
                    println!("super failed\n");
                    errors.print_errors();
                }
            } else {
                let mut actions = actions.clone();
                actions.add_set(Strategy::BruteForce, *cell, known);

                if clone.is_solved() {
                    if log {
                        println!("solved\n");
                    }
                    return Some(actions);
                }

                stack.push(Entry::new(clone, actions));
            }
        }
    }

    None
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
