use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use crate::io::{show_progress, Cancelable};
use crate::layout::{Cell, Known, KnownSet};
use crate::puzzle::{Board, ChangeResult, Changer, Strategy};
use crate::solve::find_intersection_removals;

/// Generates a complete puzzle solution.
pub struct Generator {
    rng: ThreadRng,
    shuffle: bool,
    bar: bool,
}

impl Generator {
    /// Pass true for shuffle to randomize the order the cells are solved.
    /// This will take longer and likely solve fewer cells using singles.
    pub fn new(shuffle: bool, bar: bool) -> Generator {
        Generator {
            rng: rand::thread_rng(),
            shuffle,
            bar,
        }
    }

    /// Returns a complete solution or a partial solution if canceled.
    pub fn generate(&mut self, changer: &Changer) -> Option<Board> {
        let cancelable = Cancelable::new();
        let cells = self.all_cells();
        let mut stack = Vec::with_capacity(81);
        stack.push(Entry {
            board: Board::new(),
            cell: cells[0],
            candidates: self.shuffle_candidates(KnownSet::full()),
        });

        while let Some(Entry {
            board,
            cell,
            mut candidates,
        }) = stack.pop()
        {
            if self.bar {
                show_progress(stack.len());
            }
            if cancelable.is_canceled() {
                return Some(board);
            }
            if candidates.is_empty() {
                continue;
            }

            let known = candidates.pop().unwrap();
            let mut clone = match changer.set_known(&board, Strategy::BruteForce, cell, known) {
                ChangeResult::None => {
                    // failed to set known which we know is a candidate
                    return Some(board);
                }
                ChangeResult::Valid(after, _) => *after,
                ChangeResult::Invalid(..) => {
                    continue;
                }
            };

            if let Some(effects) = find_intersection_removals(&clone, false) {
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
