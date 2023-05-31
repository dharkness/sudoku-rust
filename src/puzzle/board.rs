use std::fmt;

use crate::layout::{Cell, CellSet, House, Known, KnownSet};
use crate::solvers::deadly_rectangles::creates_deadly_rectangles;

use super::{Effects, Error, Strategy};

/// Tracks the full state of a puzzle in play.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Board {
    /// Cells that were solved at the start.
    givens: CellSet,
    /// Solved cells.
    knowns: CellSet,
    /// Values for all cells.
    values: [u8; 81],
    /// Knowns that are still possible for each cell.
    candidate_knowns: [KnownSet; 81],
    /// Cells that are still possible for each known.
    candidate_cells: [CellSet; 9],
    /// Cells that have been solved for each known.
    known_cells: [CellSet; 9],
}

impl Board {
    pub const fn new() -> Board {
        Board {
            givens: CellSet::empty(),
            knowns: CellSet::empty(),
            values: [Known::UNKNOWN; 81],
            candidate_knowns: [KnownSet::full(); 81],
            candidate_cells: [CellSet::full(); 9],
            known_cells: [CellSet::empty(); 9],
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

    pub fn all_candidates(&self, cells: CellSet) -> KnownSet {
        cells
            .iter()
            .fold(KnownSet::empty(), |acc, cell| acc | self.candidates(cell))
    }

    pub fn common_candidates(&self, cells: CellSet) -> KnownSet {
        cells
            .iter()
            .fold(KnownSet::full(), |acc, cell| acc & self.candidates(cell))
    }

    pub fn candidates(&self, cell: Cell) -> KnownSet {
        self.candidate_knowns[cell.usize()]
    }

    pub fn is_candidate(&self, cell: Cell, known: Known) -> bool {
        self.candidate_knowns[cell.usize()][known]
    }

    pub fn candidate_cells(&self, known: Known) -> CellSet {
        self.candidate_cells[known.usize()]
    }

    pub fn remove_candidate(&mut self, cell: Cell, known: Known, effects: &mut Effects) -> bool {
        let knowns = &mut self.candidate_knowns[cell.usize()];
        if knowns[known] {
            *knowns -= known;
            if knowns.is_empty() {
                effects.add_error(Error::UnsolvableCell(cell));
            } else if knowns.size() == 1 {
                effects.add_set(Strategy::NakedSingle, cell, knowns.iter().next().unwrap());
            }

            let cells = &mut self.candidate_cells[known.usize()];
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
        let all_candidates = self.candidate_cells[known.usize()];
        for house in cell.houses() {
            let house_cells = house.cells();
            if (self.known_cells[known.usize()] & house_cells).is_empty() {
                let candidates = all_candidates & house_cells;
                if candidates.is_empty() {
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

    pub fn remove_candidates(
        &mut self,
        cell: Cell,
        knowns: KnownSet,
        effects: &mut Effects,
    ) -> bool {
        knowns.iter().fold(false, |acc, known| {
            self.remove_candidate(cell, known, effects) || acc
        })
    }

    pub fn remove_many_candidate(
        &mut self,
        cells: CellSet,
        known: Known,
        effects: &mut Effects,
    ) -> bool {
        cells.iter().fold(false, |acc, cell| {
            self.remove_candidate(cell, known, effects) || acc
        })
    }

    pub fn remove_many_candidates(
        &mut self,
        cells: CellSet,
        knowns: KnownSet,
        effects: &mut Effects,
    ) -> bool {
        cells.iter().fold(false, |acc, cell| {
            knowns.iter().fold(false, |acc, known| {
                self.remove_candidate(cell, known, effects) || acc
            }) || acc
        })
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
        self.known_cells[known.usize()] += cell;
        self.candidate_cells[known.usize()] -= cell;

        let candidates = self.candidate_knowns[cell.usize()] - known;
        self.candidate_knowns[cell.usize()] = KnownSet::empty();
        for known in candidates.iter() {
            self.candidate_cells[known.usize()] -= cell;
            self.remove_candidate_cell_from_houses(cell, known, effects);
        }

        for neighbor in (self.candidate_cells[known.usize()] & cell.neighbors()).iter() {
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
