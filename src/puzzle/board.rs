use std::fmt;

use crate::layout::{Cell, CellSet, House, Known, KnownSet};
use crate::solvers::deadly_rectangles::creates_deadly_rectangles;

use super::{Effects, Error, Strategy};

/// Tracks the full state of a puzzle in play.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Board {
    givens: CellSet,
    knowns: CellSet,
    values: [u8; 81],
    known_candidates: [KnownSet; 81],
    cell_candidates: [CellSet; 9],
    cell_knowns: [CellSet; 9],
    valid: bool,
}

impl Board {
    pub const fn new() -> Board {
        Board {
            givens: CellSet::empty(),
            knowns: CellSet::empty(),
            values: [Known::UNKNOWN; 81],
            known_candidates: [KnownSet::full(); 81],
            cell_candidates: [CellSet::full(); 9],
            cell_knowns: [CellSet::empty(); 9],
            valid: true,
        }
    }

    pub fn given_count(&self) -> u8 {
        self.givens.size()
    }

    pub fn is_given(&self, cell: Cell) -> bool {
        self.givens[cell]
    }

    pub fn known_count(&self) -> u8 {
        self.knowns.size()
    }

    pub fn unknown_count(&self) -> u8 {
        81 - self.knowns.size()
    }

    pub fn is_known(&self, cell: Cell) -> bool {
        self.knowns[cell]
    }

    pub fn is_solved(&self) -> bool {
        self.knowns.is_full()
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn all_candidates(&self, cells: CellSet) -> KnownSet {
        cells
            .iter()
            .fold(KnownSet::empty(), |acc, cell| acc | self.candidates(cell))
    }

    pub fn candidates(&self, cell: Cell) -> KnownSet {
        self.known_candidates[cell.usize()]
    }

    pub fn is_candidate(&self, cell: Cell, known: Known) -> bool {
        self.known_candidates[cell.usize()][known]
    }

    pub fn remove_candidate(&mut self, cell: Cell, known: Known, effects: &mut Effects) -> bool {
        let knowns = &mut self.known_candidates[cell.usize()];
        if knowns[known] {
            // println!("remove candidate {} from {}", known, cell);
            *knowns -= known;
            if knowns.is_empty() {
                self.valid = false;
                effects.add_error(Error::UnsolvableCell(cell));
            } else if knowns.size() == 1 {
                effects.add_set(Strategy::NakedSingle, cell, knowns.iter().next().unwrap());
            }

            let cells = &mut self.cell_candidates[known.usize()];
            debug_assert!(cells[cell]);
            *cells -= cell;
            self.remove_candidate_cell_from_houses(cell, known, effects);
            true
        } else {
            false
        }
    }

    fn remove_candidate_cell_from_houses(
        &mut self,
        cell: Cell,
        known: Known,
        effects: &mut Effects,
    ) {
        let all_candidates = self.cell_candidates[known.usize()];
        for house in cell.houses() {
            let house_cells = house.cells();
            if (self.cell_knowns[known.usize()] & house_cells).is_empty() {
                let candidates = all_candidates & house_cells;
                if candidates.is_empty() {
                    self.valid = false;
                    effects.add_error(Error::UnsolvableHouse(house, known));
                } else if candidates.size() == 1 {
                    effects.add_set(
                        Strategy::HiddenSingle,
                        candidates.iter().next().unwrap(),
                        known,
                    );
                }
            }
        }
    }

    pub fn value(&self, cell: Cell) -> u8 {
        self.values[cell.usize()]
    }

    pub fn set_given(&mut self, cell: Cell, known: Known, effects: &mut Effects) -> bool {
        if self.set_known(cell, known, effects) {
            self.givens += cell;
            true
        } else {
            false
        }
    }

    pub fn set_known(&mut self, cell: Cell, known: Known, effects: &mut Effects) -> bool {
        if !self.is_candidate(cell, known) || self.is_known(cell) {
            return false;
        }

        if let Some(rectangles) = creates_deadly_rectangles(self, cell, known) {
            rectangles.into_iter().for_each(|r| {
                effects.add_error(Error::DeadlyRectangle(r));
            });
        }

        self.values[cell.usize()] = known.value();
        self.knowns += cell;
        self.cell_knowns[known.usize()] += cell;
        self.cell_candidates[known.usize()] -= cell;

        let candidates = self.known_candidates[cell.usize()] - known;
        self.known_candidates[cell.usize()] = KnownSet::empty();
        for known in candidates.iter() {
            self.cell_candidates[known.usize()] -= cell;
            self.remove_candidate_cell_from_houses(cell, known, effects);
        }

        for neighbor in (self.cell_candidates[known.usize()] & cell.neighbors()).iter() {
            effects.add_erase(Strategy::Neighbor, neighbor, known);
        }

        true
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for r in 0..9 {
            let row = House::row(r.into());
            if !first {
                write!(f, " ")?;
            }
            first = false;
            for c in 0..9 {
                let cell = row.cell(c.into());
                let value = self.value(cell);
                if value == Known::UNKNOWN {
                    write!(f, ".")?;
                } else {
                    write!(f, "{}", value)?;
                }
            }
        }
        Ok(())
    }
}
