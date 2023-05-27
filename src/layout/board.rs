use std::fmt;
use crate::effects::{Action, Actions, Effects, Error, Strategy};

use super::{Cell, CellSet, House, Known, KnownSet};

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
        cells.iter().fold(KnownSet::empty(), |acc, cell| {
            acc | self.candidates(cell)
        })
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
            assert!(cells[cell]);
            *cells -= cell;
            for house in cell.houses() {
                if (self.cell_knowns[known.usize()] & house.cells()).is_empty() {
                    let remaining = *cells & house.cells();
                    if remaining.is_empty() {
                        self.valid = false;
                        effects.add_error(Error::UnsolvableHouse(house));
                    } else if remaining.size() == 1 {
                        effects.add_set(Strategy::HiddenSingle, remaining.iter().next().unwrap(), known);
                    }
                }
            }
            true
        } else {
            false
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
        if !self.is_candidate(cell, known) {
            return false;
        }

        let current = &mut self.values[cell.usize()];
        if *current == known.value() {
            return false;
        }
        if *current != Known::UNKNOWN {
            return false;
        }

        *current = known.value();
        self.knowns += cell;
        self.cell_knowns[known.usize()] += cell;
        let candidates = self.candidates(cell) - known;
        for known in candidates.iter() {
            self.remove_candidate(cell, known, effects);
        }
        // remove without triggering errors for empty houses
        self.known_candidates[cell.usize()] -= known;
        self.cell_candidates[known.usize()] -= cell;

        for neighbor in cell.neighbors().iter() {
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
