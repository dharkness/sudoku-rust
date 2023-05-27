use std::fmt;

use super::{Cell, CellSet, House, Known, KnownSet};

/// Tracks the full state of a puzzle in play.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Board {
    givens: CellSet,
    knowns: CellSet,
    values: [u8; 81],
    candidates: [KnownSet; 81],
    valid: bool,
}

impl Board {
    pub const fn new() -> Board {
        Board {
            givens: CellSet::empty(),
            knowns: CellSet::empty(),
            values: [Known::UNKNOWN; 81],
            candidates: [KnownSet::full(); 81],
            valid: true,
        }
    }

    pub const fn given_count(&self) -> u32 {
        self.givens.size()
    }

    pub fn is_given(&self, cell: Cell) -> bool {
        self.givens[cell]
    }

    pub const fn known_count(&self) -> u32 {
        self.knowns.size()
    }

    pub fn is_known(&self, cell: Cell) -> bool {
        self.knowns[cell]
    }

    pub const fn is_solved(&self) -> bool {
        self.knowns.is_full()
    }

    pub const fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn all_candidates(&self, cells: CellSet) -> KnownSet {
        cells.iter().fold(KnownSet::empty(), |acc, cell| {
            acc | self.candidates(cell)
        })
    }

    pub const fn candidates(&self, cell: Cell) -> KnownSet {
        self.candidates[cell.usize()]
    }

    pub fn is_candidate(&self, cell: Cell, known: Known) -> bool {
        self.candidates[cell.usize()][known]
    }

    pub fn remove_candidate(&mut self, cell: Cell, known: Known) {
        let set = &mut self.candidates[cell.usize()];
        *set -= known;
        self.valid = !set.is_empty();
    }

    pub const fn value(&self, cell: Cell) -> u8 {
        self.values[cell.usize()]
    }

    pub fn set_given(&mut self, cell: Cell, known: Known) {
        self.set_known(cell, known);
        self.givens += cell;
    }

    pub fn set_known(&mut self, cell: Cell, known: Known) {
        if self.values[cell.usize()] == known.value() {
            return;
        }
        assert!(!self.is_known(cell));
        assert!(self.is_candidate(cell, known));
        self.knowns += cell;
        self.values[cell.usize()] = known.value();
        self.candidates[cell.usize()] = KnownSet::empty();

        // let mut singles: Vec<(Cell, Known)> = Vec::new();
        for cell in cell.neighbors().iter() {
            let set = &mut self.candidates[cell.usize()];
            if set[known] {
                *set -= known;
                if set.is_empty() {
                    self.valid = false;
                // } else if set.size() == 1 {
                //     singles.push((c.cell(), set.iter().next().unwrap()));
                }
            }
        }

        // for (c, k) in singles {
        //     if self.is_solved() {
        //         continue;
        //     }
        //     if !self.is_candidate(c, k) {
        //         self.set_known(c, k);
        //     } else {
        //         self.valid = false;
        //         break;
        //     }
        // }
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
