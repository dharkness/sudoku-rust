use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use crate::io::{show_progress, Cancelable};
use crate::layout::{Cell, Known, KnownSet};
use crate::puzzle::{Board, Change, Player, Strategy};
use crate::solve::find_intersection_removals;

/// Generates a complete puzzle solution.
pub struct Generator {
    rng: ThreadRng,
    shuffle: bool,
}

impl Generator {
    /// Pass true for shuffle to randomize the order the cells are solved.
    /// This will take longer and likely solve fewer cells using singles.
    pub fn new(shuffle: bool) -> Generator {
        Generator {
            rng: rand::thread_rng(),
            shuffle,
        }
    }

    /// Returns a complete solution or a partial solution if canceled.
    pub fn generate(&mut self, player: &Player, cancelable: &Cancelable) -> Option<Board> {
        let cells = self.all_cells();
        let mut stack = Vec::with_capacity(81);
        stack.push(Entry {
            board: Board::new(),
            cell: cells[0],
            candidates: self.shuffle_candidates(KnownSet::full()),
        });

        while !stack.is_empty() {
            let Entry {
                board,
                cell,
                mut candidates,
            } = stack.pop().unwrap();

            show_progress(stack.len());
            if cancelable.is_canceled() {
                return Some(board);
            }
            if candidates.is_empty() {
                continue;
            }

            let known = candidates.pop().unwrap();
            let mut clone = match player.set_known(&board, Strategy::BruteForce, cell, known) {
                Change::None => {
                    // failed to set known which we know is a candidate
                    return Some(board);
                }
                Change::Valid(after, _) => *after,
                Change::Invalid(..) => {
                    continue;
                }
            };

            if let Some(effects) = find_intersection_removals(&clone) {
                if effects.apply_all(&mut clone).is_some() {
                    continue;
                }
            }

            stack.push(Entry {
                board,
                cell,
                candidates,
            });
            loop {
                if stack.len() == 81 || cancelable.is_canceled() {
                    return Some(clone);
                }

                let next = cells[stack.len()];
                if !clone.is_known(next) {
                    stack.push(Entry {
                        board: clone,
                        cell: next,
                        candidates: self.shuffle_candidates(clone.candidates(next)),
                    });
                    break;
                }
                stack.push(Entry {
                    board: clone,
                    cell: next,
                    candidates: vec![],
                });
            }
        }

        None
    }

    fn all_cells(&mut self) -> Vec<Cell> {
        let mut cells: Vec<Cell> = Vec::with_capacity(81);

        for i in 0..81 {
            cells.push(Cell::new(i));
        }
        if self.shuffle {
            cells.shuffle(&mut self.rng);
        }

        cells
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
