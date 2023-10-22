use itertools::Itertools;
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
    set: HashMap<Cell, Known>,      // [CellSet; 9], [Value; 81]
    erase: HashMap<Cell, KnownSet>, // [CellSet; 9], [KnownSet; 81]
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

    pub fn new_erase_cells(strategy: Strategy, cells: CellSet, known: Known) -> Self {
        Self {
            strategy,
            set: HashMap::new(),
            erase: cells
                .iter()
                .map(|cell| (cell, KnownSet::empty() + known))
                .collect(),
        }
    }

    pub fn new_erase_knowns(strategy: Strategy, cell: Cell, knowns: KnownSet) -> Self {
        Self {
            strategy,
            set: HashMap::new(),
            erase: HashMap::from([(cell, knowns)]),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.set.is_empty() && self.erase.is_empty()
    }

    pub fn strategy(&self) -> Strategy {
        self.strategy
    }

    pub fn has_strategy(&self, strategy: Strategy) -> bool {
        self.strategy == strategy
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

    pub fn affects_cell(&self, cell: Cell) -> bool {
        self.erase.contains_key(&cell) || self.set.contains_key(&cell)
    }

    pub fn erases(&self, cell: Cell, known: Known) -> bool {
        match self.erase.get(&cell) {
            Some(knowns) => knowns.has(known),
            None => false,
        }
    }

    pub fn erases_from_cells(&self, known: Known) -> CellSet {
        self.erase
            .iter()
            .fold(CellSet::empty(), |cells, (cell, knowns)| {
                if knowns.has(known) {
                    cells + *cell
                } else {
                    cells
                }
            })
    }

    pub fn erases_knowns_from(&self, cell: Cell) -> KnownSet {
        self.erase[&cell]
    }

    pub fn apply(&self, board: &mut Board, effects: &mut Effects) -> bool {
        let mut changed = false;

        for (cell, knowns) in &self.erase {
            for known in knowns.iter() {
                // println!("erase {} from {}", known, cell);
                changed = board.remove_candidate(*cell, known, effects) || changed;
            }
        }

        if matches!(self.strategy, Strategy::Given) {
            for (cell, known) in &self.set {
                // println!("give {} to {}", cell, known);
                changed = board.set_given(*cell, *known, effects) || changed;
            }
        } else {
            for (cell, known) in &self.set {
                // println!("set {} to {}", cell, known);
                changed = board.set_known(*cell, *known, effects) || changed;
            }
        }

        changed
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:20}", format!("{}", self.strategy))?;
        if self.is_empty() {
            f.write_str(EMPTY_SET)
        } else {
            let mut first = true;
            for (knowns, cells) in self
                .erase
                .iter()
                .fold(
                    HashMap::new(),
                    |mut map: HashMap<KnownSet, CellSet>, (cell, knowns)| {
                        *map.entry(*knowns).or_default() += *cell;
                        map
                    },
                )
                .iter()
                .sorted_by(|(_, a), (_, b)| b.len().cmp(&a.len()))
            {
                if first {
                    first = false;
                } else {
                    f.write_str(", ")?;
                }
                for known in knowns.iter() {
                    f.write_char(known.label())?;
                }
                write!(f, " {} {}", REMOVE_CANDIDATE, cells)?;
            }
            for (known, cells) in self
                .set
                .iter()
                .fold(
                    HashMap::new(),
                    |mut map: HashMap<Known, CellSet>, (cell, known)| {
                        *map.entry(*known).or_default() += *cell;
                        map
                    },
                )
                .iter()
                .sorted_by(|(a, _), (b, _)| a.cmp(b))
            {
                if first {
                    first = false;
                } else {
                    f.write_str(", ")?;
                }
                write!(f, "{} {} {}", known, SET_KNOWN, cells)?;
            }
            Ok(())
        }
    }
}
