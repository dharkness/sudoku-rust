use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use crate::layout::{Board, Cell, CellSet, Known, KnownSet};
use crate::printers::{print_candidates, print_values};

const FILLED: &str = "|---------=========---------=========---------=========---------=========---------|";
const EMPTY : &str = "|                                                                                 |";

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
        let mut stack = Vec::with_capacity(81);

        stack.push(Entry {
            board: Board::new(),
            cell: self.cells[0],
            candidates: self.shuffle_candidates(KnownSet::full()),
        });

        while !stack.is_empty() {
            println!("{}{}", FILLED[..stack.len()+1].to_string(), EMPTY[stack.len()+1..].to_string());

            let Entry { board, cell, mut candidates } = stack.pop().unwrap();
            if candidates.is_empty() {
                continue;
            }
            // if stack.len() % 10 == 0 {
            //     print_values(&board);
            //     print_candidates(&board);
            //     println!("{}: {:?}", cell.label(), candidates);
            // }

            let candidate = candidates.pop().unwrap();
            let mut clone = board.clone();
            clone.set_known(cell, candidate);
            stack.push(Entry {
                board,
                cell,
                candidates,
            });

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
            print_candidates(&board);
            println!("Board: {}", board);
        },
        None => {
            println!("No solution found");
        },
    }
}
