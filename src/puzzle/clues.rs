use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Write;

use colored::Colorize;

use crate::layout::{Cell, CellSet, Known, KnownSet};
use crate::symbols::EMPTY_SET;

#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Verdict {
    #[default]
    None,
    Set,
    Erase,
    Related,
    Primary,
    Secondary,
    Tertiary,
}

impl Verdict {
    pub fn color_char(self, c: char) -> String {
        self.color(c.to_string())
    }

    pub fn color(self, str: String) -> String {
        match self {
            Self::None => str,
            Self::Set => str.bright_green().bold().blink().to_string(),
            Self::Erase => str.bright_yellow().bold().blink().to_string(),
            Self::Related => str.bright_blue().bold().blink().to_string(),
            Self::Primary => str.bright_purple().bold().blink().to_string(),
            Self::Secondary => str.bright_cyan().bold().blink().to_string(),
            Self::Tertiary => str.bright_red().bold().blink().to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Clue {
    verdict: Verdict,
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

    pub fn clue_cell_for_known(&mut self, color: Verdict, cell: Cell, known: Known) {
        self.clue_cells_for_known(color, CellSet::empty() + cell, known)
    }

    pub fn clue_cells_for_known(&mut self, color: Verdict, cells: CellSet, known: Known) {
        let clue = Clue {
            verdict: color,
            known,
            cells,
        };
        match self.clues.binary_search_by(|clue| {
            match color.partial_cmp(&clue.verdict) {
                Some(Ordering::Equal) => known.partial_cmp(&clue.known),
                result => result,
            }
            .unwrap()
        }) {
            Ok(index) => self.clues[index].cells |= cells,
            Err(index) => self.clues.insert(index, clue),
        }
    }

    pub fn clue_cell_for_knowns(&mut self, color: Verdict, cell: Cell, knowns: KnownSet) {
        self.clue_cells_for_knowns(color, CellSet::empty() + cell, knowns)
    }

    pub fn clue_cells_for_knowns(&mut self, color: Verdict, cells: CellSet, knowns: KnownSet) {
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

    pub fn collect(&self) -> HashMap<Cell, HashMap<Known, Verdict>> {
        self.clues.iter().fold(HashMap::new(), |mut map, clue| {
            clue.cells.iter().for_each(|cell| {
                map.entry(cell)
                    .or_default()
                    .insert(clue.known, clue.verdict);
            });
            map
        })
    }

    pub fn collect_for_known(&self, known: Known) -> HashMap<Cell, Verdict> {
        self.clues.iter().filter(|clue| clue.known == known).fold(
            HashMap::new(),
            |mut map, clue| {
                clue.cells.iter().for_each(|cell| {
                    map.insert(cell, clue.verdict);
                });
                map
            },
        )
    }
}

impl fmt::Display for Clues {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            f.write_char(EMPTY_SET)
        } else {
            let mut first = true;
            let mut prev_color = Verdict::Secondary;
            for Clue {
                verdict: color,
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
