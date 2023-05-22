use std::fmt;

use super::{Cell, CellSet, Coord, House, Known, KnownSet};

const UNKNOWN: u8 = 0;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Board {
    givens: CellSet,
    knowns: CellSet,
    values: [u8; 81],
    candidates: [KnownSet; 81],
}

impl Board {
    pub fn new() -> Board {
        Board {
            givens: CellSet::empty(),
            knowns: CellSet::empty(),
            values: [UNKNOWN; 81],
            candidates: [KnownSet::full(); 81],
        }
    }

    pub fn given_count(&self) -> u32 {
        self.givens.size()
    }

    pub fn is_given(&self, cell: Cell) -> bool {
        self.givens[cell]
    }

    pub fn known_count(&self) -> u32 {
        self.knowns.size()
    }

    pub fn is_known(&self, cell: Cell) -> bool {
        self.knowns[cell]
    }

    pub fn is_solved(&self) -> bool {
        self.knowns.is_full()
    }

    pub fn candidates(&self, cell: Cell) -> KnownSet {
        self.candidates[cell.index() as usize]
    }

    pub fn is_candidate(&self, cell: Cell, known: Known) -> bool {
        self.candidates[cell.index() as usize][known]
    }

    pub fn remove_candidate(&mut self, cell: Cell, known: Known) {
        assert!(self.is_candidate(cell, known));
        self.candidates[cell.index() as usize] -= known;
    }

    pub fn value(&self, cell: Cell) -> u8 {
        self.values[cell.index() as usize]
    }

    pub fn set_given(&mut self, cell: Cell, known: Known) {
        self.set_known(cell, known);
        self.givens += cell;
    }

    pub fn set_known(&mut self, cell: Cell, known: Known) {
        assert!(!self.is_known(cell));
        assert!(self.is_candidate(cell, known));
        self.knowns += cell;
        self.values[cell.usize()] = known.value();
        self.candidates[cell.usize()] = KnownSet::empty();
        for c in cell.neighbors().iter() {
            self.candidates[c.cell().usize()] -= known;
        }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for r in 0..9 {
            let row = House::row(Coord::new(r as u8));
            if !first {
                write!(f, " ")?;
            }
            first = false;
            for c in 0..9 {
                let cell = row.cell(Coord::new(c as u8));
                let value = self.value(cell);
                if value == UNKNOWN {
                    write!(f, ".")?;
                } else {
                    write!(f, "{}", value)?;
                }
            }
        }
        Ok(())
    }
}
