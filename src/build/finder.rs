use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use crate::io::{show_progress, Cancelable};
use crate::layout::{Cell, CellSet};
use crate::puzzle::{Board, Effects, Options, Player};
use crate::solve::{Resolution, Solver};

/// Finds a solvable starting puzzle from a full solution.
pub struct Finder {
    rng: ThreadRng,
    clues: usize,
    time: u64,
    bar: bool,
}

impl Finder {
    pub fn new(clues: usize, time: u64, bar: bool) -> Finder {
        Finder {
            rng: rand::thread_rng(),
            clues,
            time,
            bar,
        }
    }

    pub fn backtracking_find(&mut self, board: Board, cancelable: &Cancelable) -> (Board, Effects) {
        let player = Player::new(Options::errors_and_peers());
        let solver = Solver::new(cancelable, false);
        let runtime = std::time::Instant::now();

        let mut fewest_clues = 81;
        let mut fewest_clues_board = board;
        let mut fewest_clues_actions = Effects::new();

        let mut stack = Vec::with_capacity(81);
        stack.push(Entry {
            board,
            cells: self.shuffle_cells(board.knowns()),
        });

        while !stack.is_empty() {
            if self.bar {
                show_progress(82 - stack.len());
            }
            if cancelable.is_canceled()
                || fewest_clues <= self.clues
                || runtime.elapsed().as_secs() >= self.time
            {
                break;
            }

            let entry = stack.last_mut().unwrap();
            if entry.cells.is_empty() {
                stack.pop();
                continue;
            }

            let cell = entry.cells.pop().unwrap();
            let (next, unapplied) = player.without(&entry.board, cell);

            match solver.solve(&next, &unapplied) {
                Resolution::Canceled(..) => break,
                Resolution::Solved(_, actions, _) => {
                    if next.known_count() < fewest_clues {
                        fewest_clues = next.known_count();
                        fewest_clues_board = next;
                        fewest_clues_actions = actions;
                    }
                    stack.push(Entry {
                        board: next,
                        cells: self.shuffle_cells(next.knowns()),
                    });
                }
                _ => continue,
            }
        }

        (fewest_clues_board, fewest_clues_actions)
    }

    fn shuffle_cells(&mut self, set: CellSet) -> Vec<Cell> {
        let mut cells = set.iter().collect::<Vec<Cell>>();

        cells.shuffle(&mut self.rng);

        cells
    }
}

struct Entry {
    board: Board,
    cells: Vec<Cell>,
}
