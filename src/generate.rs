use std::sync::atomic::{AtomicBool, Ordering};

use ctrlc;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use crate::layout::{Board, Cell, Known, KnownSet};
use crate::printers::{print_candidates, print_values};
use crate::solvers::deadly_rectangles::creates_deadly_rectangle;
use crate::solvers::intersection_removals::find_intersection_removals;

const FILLED: &str = "|---------=========---------=========---------=========---------=========---------|";
const EMPTY : &str = "|                                                                                 |";

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
        // cells.shuffle(&mut rng);

        Generator { rng, cells }
    }

    pub fn generate(&mut self) -> Option<Board> {
        static CANCEL: AtomicBool = AtomicBool::new(false);
        ctrlc::set_handler(|| CANCEL.store(true, Ordering::Relaxed)).expect("Error setting Ctrl-C handler");

        let mut stack = Vec::with_capacity(81);

        stack.push(Entry {
            board: Board::new(),
            cell: self.cells[0],
            candidates: self.shuffle_candidates(KnownSet::full()),
        });

        while !stack.is_empty() {
            println!("{}{}", FILLED[..stack.len()+1].to_string(), EMPTY[stack.len()+1..].to_string());

            let Entry { board, cell, mut candidates } = stack.pop().unwrap();
            if CANCEL.load(Ordering::Relaxed) {
                return Some(board);
            }
            if candidates.is_empty() {
                continue;
            }
            // if stack.len() % 10 == 0 {
            //     print_values(&board);
            //     print_candidates(&board);
            //     println!("{}: {:?}", cell.label(), candidates.iter().map(|k| k.label()).collect::<Vec<&str>>());
            // }

            let candidate = candidates.pop().unwrap();
            if stack.len() >= 3 && creates_deadly_rectangle(&board, cell, candidate) {
                continue;
            }
            let mut clone = board.clone();
            clone.set_known(cell, candidate);
            let remove = find_intersection_removals(&clone);
            if remove.len() > 0 {
                // print_candidates(&clone);
                // println!("{:?}", remove.iter().map(|(c, k)| (c.label(), k.label())).collect::<Vec<(&str, &str)>>());
                for (c, k) in remove {
                    clone.remove_candidate(c, k);
                }
            }
            if !clone.is_valid() {
                continue;
            }

            stack.push(Entry { board, cell, candidates });
            if stack.len() == 81 {
                return Some(clone);
            }

            let next = self.cells[stack.len()];
            stack.push(Entry {
                board: clone,
                cell: next,
                candidates: self.shuffle_candidates(clone.candidates(next)),
            });
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

pub fn generate_board() {
    let mut generator = Generator::new();

    match generator.generate() {
        Some(board) => {
            print_values(&board);
            println!("Board: {}", board);
        },
        None => {
            println!("No solution found");
        },
    }
}
