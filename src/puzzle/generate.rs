use std::sync::atomic::{AtomicBool, Ordering};

use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use crate::layout::{Cell, Known, KnownSet};
use crate::printers::{print_candidates, print_values};
use crate::solvers::intersection_removals::find_intersection_removals;

use super::{Board, Effects};

const FILLED: &str =
    "|---------=========---------=========---------=========---------=========---------|";
const EMPTY: &str =
    "|                                                                                 |";

/// Generates a full board.
pub struct Generator {
    rng: ThreadRng,
    cells: Vec<Cell>,
}

impl Generator {
    pub fn new() -> Generator {
        let mut rng = rand::thread_rng();
        let mut cells: Vec<Cell> = Vec::with_capacity(81);

        for i in 0..81 {
            cells.push(Cell::new(i));
        }
        cells.shuffle(&mut rng);

        Generator { rng, cells }
    }

    pub fn generate(&mut self) -> Option<Board> {
        static CANCEL: AtomicBool = AtomicBool::new(false);
        ctrlc::set_handler(|| CANCEL.store(true, Ordering::Relaxed))
            .expect("Error setting Ctrl-C handler");

        let mut stack = Vec::with_capacity(81);
        stack.push(Entry {
            board: Board::new(),
            cell: self.cells[0],
            candidates: self.shuffle_candidates(KnownSet::full()),
        });

        while !stack.is_empty() {
            println!(
                "{}{}",
                &FILLED[..stack.len() + 1],
                &EMPTY[stack.len() + 1..]
            );

            let Entry {
                board,
                cell,
                mut candidates,
            } = stack.pop().unwrap();
            if CANCEL.load(Ordering::Relaxed) {
                return Some(board);
            }

            // print_candidates(&board);
            // println!("stack size {}, cell {}, candidates {:?}", stack.len(), cell, candidates);
            if candidates.is_empty() {
                // println!("{} is unsolvable {}", cell.label(), board.candidates(cell));
                continue;
            }
            // if stack.len() % 10 == 0 {
            //     print_values(&board);
            //     print_candidates(&board);
            //     println!("{}: {:?}", cell.label(), candidates.iter().map(|k| k.label()).collect::<Vec<&str>>());
            // }

            let candidate = candidates.pop().unwrap();
            let mut clone = board;
            let mut effects = Effects::new();
            clone.set_known(cell, candidate, &mut effects);
            if effects.apply_all(&mut clone).is_some() {
                // print_candidates(&clone);
                // println!("intersection removals caused errors");
                continue;
            }

            if let Some(effects) = find_intersection_removals(&clone) {
                if effects.apply_all(&mut clone).is_some() {
                    // print_candidates(&clone);
                    // println!("intersection removals caused errors");
                    continue;
                }
            }
            // if !clone.is_valid() {
            //     println!("invalid with error");
            //     continue;
            // }

            stack.push(Entry {
                board,
                cell,
                candidates,
            });
            loop {
                if stack.len() == 81 {
                    return Some(clone);
                }

                let next = self.cells[stack.len()];
                if !clone.is_known(next) {
                    // println!("next {} candidates {}", next, clone.candidates(next));
                    stack.push(Entry {
                        board: clone,
                        cell: next,
                        candidates: self.shuffle_candidates(clone.candidates(next)),
                    });
                    break;
                }
                // println!("{} is solved", next);
                stack.push(Entry {
                    board: clone,
                    cell: next,
                    candidates: vec![],
                });
            }
        }

        None
    }

    fn shuffle_candidates(&mut self, candidates: KnownSet) -> Vec<Known> {
        let mut shuffled = candidates.iter().collect::<Vec<Known>>();
        shuffled.shuffle(&mut self.rng);
        shuffled
    }
}

struct Entry {
    board: Board,
    cell: Cell,
    candidates: Vec<Known>,
}

#[allow(dead_code)]
pub fn generate_board() {
    let mut generator = Generator::new();

    match generator.generate() {
        Some(board) => {
            print_values(&board);
            if !board.is_solved() {
                print_candidates(&board);
            }
            println!("Board: {}", board);
        }
        None => {
            println!("No solution found");
        }
    }
}
