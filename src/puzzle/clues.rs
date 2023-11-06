use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Write;

use crate::layout::{Cell, CellSet, Known, KnownSet};
use crate::symbols::EMPTY_SET;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Color {
    None,
    Blue,
    Green,
    Purple,
    Red,
    Yellow,
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Clue {
    color: Color,
    known: Known,
    cells: CellSet,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Clues {
    clues: Vec<Clue>,
}

impl Clues {
    pub const fn new() -> Self {
        Self { clues: Vec::new() }
    }

    pub fn clue_cell_for_known(&mut self, color: Color, cell: Cell, known: Known) {
        self.clue_cells_for_known(color, CellSet::empty() + cell, known)
    }

    pub fn clue_cells_for_known(&mut self, color: Color, cells: CellSet, known: Known) {
        let clue = Clue {
            color,
            known,
            cells,
        };
        match self.clues.binary_search_by(|clue| {
            match color.partial_cmp(&clue.color) {
                Some(Ordering::Equal) => known.partial_cmp(&clue.known),
                result => result,
            }
            .unwrap()
        }) {
            Ok(index) => self.clues[index].cells |= cells,
            Err(index) => self.clues.insert(index, clue),
        }
    }

    pub fn clue_cell_for_knowns(&mut self, color: Color, cell: Cell, knowns: KnownSet) {
        self.clue_cells_for_knowns(color, CellSet::empty() + cell, knowns)
    }

    pub fn clue_cells_for_knowns(&mut self, color: Color, cells: CellSet, knowns: KnownSet) {
        knowns
            .iter()
            .for_each(|known| self.clue_cells_for_known(color, cells, known))
    }

    pub fn is_empty(&self) -> bool {
        self.clues.is_empty()
    }

    pub fn clues(&self) -> &Vec<Clue> {
        &self.clues
    }

    pub fn collect(&self) -> HashMap<Cell, HashMap<Known, Color>> {
        self.clues.iter().fold(HashMap::new(), |mut map, clue| {
            clue.cells.iter().for_each(|cell| {
                map.entry(cell).or_default().insert(clue.known, clue.color);
            });
            map
        })
    }
}

impl fmt::Display for Clues {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            f.write_char(EMPTY_SET)
        } else {
            let mut first = true;
            let mut prev_color = Color::Blue;
            for Clue {
                color,
                known,
                cells,
            } in &self.clues
            {
                if first {
                    first = false;
                    write!(f, "{:?} [", *color)?;
                } else if *color != prev_color {
                    write!(f, "] {:?} [", *color)?;
                    prev_color = *color;
                } else {
                    f.write_str(", ")?;
                }
                write!(f, "{}: {}", known, cells)?;
            }
            write!(f, "]")
        }
    }
}
