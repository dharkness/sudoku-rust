use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Write;
use std::vec::IntoIter;

use itertools::Itertools;

use crate::layout::{Cell, CellSet, Digit, DigitSet};
use crate::symbols::{EMPTY_SET, REMOVE_CANDIDATE, SET_DIGIT};

use super::{Board, Change, Clues, Effects, Strategy, Verdict};

/// One or more changes to the board derived using a specific strategy.
#[derive(Clone, Eq, PartialEq)]
pub struct Action {
    strategy: Strategy,
    set: HashMap<Cell, Digit>,      // [CellSet; 9], [Value; 81]
    erase: HashMap<Cell, DigitSet>, // [CellSet; 9], [DigitSet; 81]
    clues: Clues,
}

impl Action {
    pub fn new(strategy: Strategy) -> Self {
        Self {
            strategy,
            set: HashMap::new(),
            erase: HashMap::new(),
            clues: Clues::new(),
        }
    }

    pub fn new_set(strategy: Strategy, cell: Cell, digit: Digit) -> Self {
        Self {
            strategy,
            set: HashMap::from([(cell, digit)]),
            erase: HashMap::new(),
            clues: Clues::new(),
        }
    }

    pub fn new_erase(strategy: Strategy, cell: Cell, digit: Digit) -> Self {
        Self {
            strategy,
            set: HashMap::new(),
            erase: HashMap::from([(cell, DigitSet::of(digit))]),
            clues: Clues::new(),
        }
    }

    pub fn new_erase_cells(strategy: Strategy, cells: CellSet, digit: Digit) -> Self {
        Self {
            strategy,
            set: HashMap::new(),
            erase: cells
                .iter()
                .map(|cell| (cell, DigitSet::of(digit)))
                .collect(),
            clues: Clues::new(),
        }
    }

    pub fn new_erase_digits(strategy: Strategy, cell: Cell, digits: DigitSet) -> Self {
        Self {
            strategy,
            set: HashMap::new(),
            erase: HashMap::from([(cell, digits)]),
            clues: Clues::new(),
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

    pub fn set(&mut self, cell: Cell, digit: Digit) {
        self.set.insert(cell, digit);
    }

    pub fn sets(&self, cell: Cell, digit: Digit) -> bool {
        match self.set.get(&cell) {
            Some(d) => *d == digit,
            None => false,
        }
    }

    pub fn collect_sets(&self) -> IntoIter<(Cell, Digit)> {
        self.set
            .iter()
            .map(|(cell, digit)| (*cell, *digit))
            .sorted_by(|a, b| match a.0.cmp(&b.0) {
                Ordering::Equal => a.1.cmp(&b.1),
                result => result,
            })
    }

    pub fn erase(&mut self, cell: Cell, digit: Digit) {
        *self.erase.entry(cell).or_insert_with(DigitSet::empty) += digit;
    }

    pub fn erase_cells(&mut self, cells: CellSet, digit: Digit) {
        cells.iter().for_each(|cell| self.erase(cell, digit));
    }

    pub fn erase_digits(&mut self, cell: Cell, digits: DigitSet) {
        digits.iter().for_each(|digit| self.erase(cell, digit));
    }

    pub fn affects_cell(&self, cell: Cell) -> bool {
        self.erase.contains_key(&cell) || self.set.contains_key(&cell)
    }

    pub fn affects_digit(&self, digit: Digit) -> bool {
        self.erase.values().any(|ds| ds.has(digit)) || self.set.values().any(|d| *d == digit)
    }

    pub fn erases(&self, cell: Cell, digit: Digit) -> bool {
        match self.erase.get(&cell) {
            Some(digits) => digits.has(digit),
            None => false,
        }
    }

    pub fn erases_from_cells(&self, digit: Digit) -> CellSet {
        self.erase
            .iter()
            .fold(CellSet::empty(), |cells, (cell, digits)| {
                if digits.has(digit) {
                    cells + *cell
                } else {
                    cells
                }
            })
    }

    pub fn erases_digits_from(&self, cell: Cell) -> DigitSet {
        self.erase[&cell]
    }

    pub fn collect_erases(&self) -> IntoIter<(Cell, DigitSet)> {
        self.erase
            .iter()
            .map(|(cell, digits)| (*cell, *digits))
            .sorted_by(|a, b| match a.0.cmp(&b.0) {
                Ordering::Equal => a.1.cmp(&b.1),
                result => result,
            })
    }

    pub fn clue_cell_for_digit(&mut self, color: Verdict, cell: Cell, digit: Digit) {
        self.clues.clue_cell_for_digit(color, cell, digit);
    }

    pub fn clue_cells_for_digit(&mut self, color: Verdict, cells: CellSet, digit: Digit) {
        self.clues.clue_cells_for_digit(color, cells, digit);
    }

    pub fn clue_cell_for_digits(&mut self, color: Verdict, cell: Cell, digits: DigitSet) {
        self.clues.clue_cell_for_digits(color, cell, digits);
    }

    pub fn clue_cells_for_digits(&mut self, color: Verdict, cells: CellSet, digits: DigitSet) {
        self.clues.clue_cells_for_digits(color, cells, digits);
    }

    pub fn has_clues(&self) -> bool {
        !self.clues.is_empty()
    }

    pub fn clues(&self) -> &Clues {
        &self.clues
    }

    pub fn collect_clues(&self) -> IntoIter<(Cell, Digit, Verdict)> {
        self.clues
            .collect()
            .iter()
            .flat_map(|(cell, map)| map.iter().map(|(digit, color)| (*cell, *digit, *color)))
            .sorted_by(|a, b| match a.0.cmp(&b.0) {
                Ordering::Equal => match a.1.cmp(&b.1) {
                    Ordering::Equal => a.2.cmp(&b.2),
                    result => result,
                },
                result => result,
            })
    }

    pub fn collect_verdicts(&self) -> HashMap<Cell, HashMap<Digit, Verdict>> {
        let mut verdicts = self.clues.collect();
        for (cell, digits) in &self.erase {
            let map = verdicts.entry(*cell).or_default();
            for digit in *digits {
                map.insert(digit, Verdict::Erase);
            }
        }
        for (cell, digit) in &self.set {
            verdicts
                .entry(*cell)
                .or_default()
                .insert(*digit, Verdict::Set);
        }
        verdicts
    }

    pub fn collect_verdicts_for_digit(&self, digit: Digit) -> HashMap<Cell, Verdict> {
        let mut verdicts = self.clues.collect_for_digit(digit);
        for (cell, digits) in &self.erase {
            if digits.has(digit) {
                verdicts.insert(*cell, Verdict::Erase);
            }
        }
        for (cell, _) in self.set.iter().filter(|(_, d)| **d == digit) {
            verdicts.insert(*cell, Verdict::Set);
        }
        verdicts
    }

    pub fn apply(&self, board: &mut Board, effects: &mut Effects) -> Change {
        let mut change = Change::None;

        for (cell, digits) in &self.erase {
            for digit in digits.iter() {
                // println!("erase {} from {}", digit, cell);
                change &= board.remove_candidate(*cell, digit, effects);
            }
        }

        if matches!(self.strategy, Strategy::Given) {
            for (cell, digit) in &self.set {
                // println!("give {} to {}", cell, digit);
                change &= board.set_given(*cell, *digit, effects);
            }
        } else {
            for (cell, digit) in &self.set {
                // println!("set {} to {}", cell, digit);
                change &= board.set_digit(*cell, *digit, effects);
            }
        }

        change
    }
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.strategy)?;
        if self.is_empty() {
            f.write_char(' ')?;
            f.write_char(EMPTY_SET)
        } else {
            for (cell, digits) in self.collect_erases() {
                f.write_str(&format!("\n- {} {} {}", cell, REMOVE_CANDIDATE, digits))?;
            }
            for (cell, digit) in self.collect_sets() {
                f.write_str(&format!("\n- {} {} {}", cell, SET_DIGIT, digit))?;
            }
            for (cell, digit, color) in self.collect_clues() {
                f.write_str(&format!("\n- {} {} {:?}", cell, digit, color))?;
            }
            Ok(())
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:30}", format!("{}", self.strategy))?;
        if self.is_empty() {
            f.write_char(EMPTY_SET)
        } else {
            let mut first = true;
            for (digits, cells) in self
                .erase
                .iter()
                .fold(
                    HashMap::new(),
                    |mut map: HashMap<DigitSet, CellSet>, (cell, digits)| {
                        *map.entry(*digits).or_default() += *cell;
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
                for digit in digits.iter() {
                    f.write_char(digit.label())?;
                }
                write!(f, " {} {}", REMOVE_CANDIDATE, cells)?;
            }
            for (digit, cells) in self
                .set
                .iter()
                .fold(
                    HashMap::new(),
                    |mut map: HashMap<Digit, CellSet>, (cell, digit)| {
                        *map.entry(*digit).or_default() += *cell;
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
                write!(f, "{} {} {}", digit, SET_DIGIT, cells)?;
            }
            Ok(())
        }
    }
}
