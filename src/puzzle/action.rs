use std::collections::HashMap;
use std::fmt;

use crate::layout::{Cell, Known, KnownSet};

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
            write!(f, "∅")
        } else {
            let mut first = true;
            for (cell, knowns) in &self.erase {
                if first {
                    first = false;
                } else {
                    write!(f, ", ")?;
                }
                write!(f, "{} × ", cell)?;
                for known in knowns.iter() {
                    write!(f, "{}", known)?;
                }
            }
            for (cell, known) in &self.set {
                if first {
                    first = false;
                } else {
                    write!(f, ", ")?;
                }
                write!(f, "{} ⇨ {}", cell, known)?;
            }
            Ok(())
        }
    }
}
