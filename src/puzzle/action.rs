use std::collections::HashMap;
use std::fmt;
use std::fmt::Write;

use crate::layout::{Cell, CellSet, Known, KnownSet};
use crate::symbols::{EMPTY_SET, REMOVE_CANDIDATE, SET_KNOWN};

use super::{Board, Effects, Strategy};

/// One or more changes to the board derived using a specific strategy.
#[derive(Clone, Debug)]
pub struct Action {
    strategy: Strategy,
    set: HashMap<Cell, Known>,
    erase: HashMap<Cell, KnownSet>,
}

impl Action {
    pub fn new(strategy: Strategy) -> Self {
        Self {
            strategy,
            set: HashMap::new(),
            erase: HashMap::new(),
        }
    }

    pub fn new_set(strategy: Strategy, cell: Cell, known: Known) -> Self {
        Self {
            strategy,
            set: HashMap::from([(cell, known)]),
            erase: HashMap::new(),
        }
    }

    pub fn new_erase(strategy: Strategy, cell: Cell, known: Known) -> Self {
        Self {
            strategy,
            set: HashMap::new(),
            erase: HashMap::from([(cell, KnownSet::empty() + known)]),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.set.is_empty() && self.erase.is_empty()
    }

    pub fn set(&mut self, cell: Cell, known: Known) {
        self.set.insert(cell, known);
    }

    pub fn erase(&mut self, cell: Cell, known: Known) {
        *self.erase.entry(cell).or_insert_with(KnownSet::empty) += known;
    }

    pub fn erase_cells(&mut self, cells: CellSet, known: Known) {
        cells.iter().for_each(|cell| self.erase(cell, known));
    }

    pub fn erase_knowns(&mut self, cell: Cell, knowns: KnownSet) {
        knowns.iter().for_each(|known| self.erase(cell, known));
    }

    pub fn apply(&self, board: &mut Board, effects: &mut Effects) {
        for (cell, knowns) in &self.erase {
            for known in knowns.iter() {
                // println!("erase {} from {}", known, cell);
                board.remove_candidate(*cell, known, effects);
            }
        }
        for (cell, known) in &self.set {
            // println!("set {} to {}", cell, known);
            board.set_known(*cell, *known, effects);
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            f.write_str(EMPTY_SET)
        } else {
            let mut first = true;
            for (cell, knowns) in &self.erase {
                if first {
                    first = false;
                } else {
                    f.write_str(", ")?;
                }
                write!(f, "{} {} ", cell, REMOVE_CANDIDATE)?;
                for known in knowns.iter() {
                    f.write_char(known.label())?;
                }
            }
            for (cell, known) in &self.set {
                if first {
                    first = false;
                } else {
                    f.write_str(", ")?;
                }
                write!(f, "{} {} {}", cell, SET_KNOWN, known)?;
            }
            Ok(())
        }
    }
}
