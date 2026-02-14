use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Write;

use colored::Colorize;

use crate::layout::{Cell, CellSet, Digit, DigitSet};
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
    digit: Digit,
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

    pub fn clue_cell_for_digit(&mut self, color: Verdict, cell: Cell, digit: Digit) {
        self.clue_cells_for_digit(color, CellSet::empty() + cell, digit)
    }

    pub fn clue_cells_for_digit(&mut self, color: Verdict, cells: CellSet, digit: Digit) {
        let clue = Clue {
            verdict: color,
            digit,
            cells,
        };
        match self.clues.binary_search_by(|clue| {
            match color.partial_cmp(&clue.verdict) {
                Some(Ordering::Equal) => digit.partial_cmp(&clue.digit),
                result => result,
            }
            .unwrap()
        }) {
            Ok(index) => self.clues[index].cells |= cells,
            Err(index) => self.clues.insert(index, clue),
        }
    }

    pub fn clue_cell_for_digits(&mut self, color: Verdict, cell: Cell, digits: DigitSet) {
        self.clue_cells_for_digits(color, CellSet::empty() + cell, digits)
    }

    pub fn clue_cells_for_digits(&mut self, color: Verdict, cells: CellSet, digits: DigitSet) {
        digits
            .iter()
            .for_each(|digit| self.clue_cells_for_digit(color, cells, digit))
    }

    pub fn is_empty(&self) -> bool {
        self.clues.is_empty()
    }

    pub fn clues(&self) -> &Vec<Clue> {
        &self.clues
    }

    pub fn collect(&self) -> HashMap<Cell, HashMap<Digit, Verdict>> {
        self.clues.iter().fold(HashMap::new(), |mut map, clue| {
            clue.cells.iter().for_each(|cell| {
                map.entry(cell)
                    .or_default()
                    .insert(clue.digit, clue.verdict);
            });
            map
        })
    }

    pub fn collect_for_digit(&self, digit: Digit) -> HashMap<Cell, Verdict> {
        self.clues.iter().filter(|clue| clue.digit == digit).fold(
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
                digit,
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
                write!(f, "{}: {}", digit, cells)?;
            }
            write!(f, "]")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbols::EMPTY_SET;
    use crate::*;

    #[test]
    fn new_is_empty() {
        let clues = Clues::new();

        assert!(clues.is_empty());
        assert!(clues.clues().is_empty());
        assert_eq!(EMPTY_SET.to_string(), format!("{}", clues));
    }

    #[test]
    fn clue_cells_merge_by_digit() {
        let mut clues = Clues::new();
        clues.clue_cell_for_digit(Verdict::Primary, cell!(A1), digit!(3));
        clues.clue_cell_for_digit(Verdict::Primary, cell!(B2), digit!(3));

        assert_eq!(1, clues.clues().len());
        assert_eq!(cells![A1 B2], clues.clues()[0].cells);
        assert_eq!(digit!(3), clues.clues()[0].digit);
        assert_eq!(Verdict::Primary, clues.clues()[0].verdict);
    }

    #[test]
    fn clue_cells_for_digits_adds_each_digit() {
        let mut clues = Clues::new();
        clues.clue_cells_for_digits(Verdict::Secondary, cells![A1], digits![1 2]);

        let digits = clues
            .clues()
            .iter()
            .fold(DigitSet::empty(), |acc, clue| acc + clue.digit);

        assert_eq!(digits![1 2], digits);
    }

    #[test]
    fn collect_maps_cells_and_digits() {
        let mut clues = Clues::new();
        clues.clue_cell_for_digit(Verdict::Primary, cell!(A1), digit!(3));
        clues.clue_cells_for_digit(Verdict::Secondary, cells![B2 C3], digit!(4));

        let map = clues.collect();
        assert_eq!(Verdict::Primary, map[&cell!(A1)][&digit!(3)]);
        assert_eq!(Verdict::Secondary, map[&cell!(B2)][&digit!(4)]);
        assert_eq!(Verdict::Secondary, map[&cell!(C3)][&digit!(4)]);

        let by_digit = clues.collect_for_digit(digit!(4));
        assert_eq!(Verdict::Secondary, by_digit[&cell!(B2)]);
    }

    #[test]
    fn verdict_color_includes_char() {
        let plain = Verdict::None.color_char('x');
        let colored = Verdict::Primary.color_char('x');

        assert_eq!("x", plain);
        assert!(colored.contains('x'));
        assert!(!colored.is_empty());
    }

    #[test]
    fn display_includes_verdicts() {
        let mut clues = Clues::new();
        clues.clue_cell_for_digit(Verdict::Primary, cell!(A1), digit!(3));

        let text = format!("{}", clues);

        assert!(text.contains("Primary"));
        assert!(text.contains("3"));
        assert!(text.contains("A1"));
    }
}
