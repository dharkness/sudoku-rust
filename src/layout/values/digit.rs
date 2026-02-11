use std::fmt;
use std::ops::{Add, Neg};
use std::str::FromStr;

use super::{DigitSet, Value};

/// Holds one of the possible digit values.
#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Digit(u8);

/// Errors that can occur when parsing a digit value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DigitError {
    InvalidValue(String),
}

impl fmt::Display for DigitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DigitError::InvalidValue(label) => {
                write!(f, "invalid digit '{}'", label)
            }
        }
    }
}

impl std::error::Error for DigitError {}

impl Digit {
    pub const COUNT: u8 = 9;

    pub fn iter() -> DigitIter {
        DigitIter::new()
    }

    pub const fn new(index: u32) -> Self {
        debug_assert!(index < 9);
        Self(index as u8)
    }

    pub const fn from_ordinal(digit: u8) -> Self {
        debug_assert!(1 <= digit && digit <= 9);
        Self(digit - 1)
    }

    pub const fn usize(&self) -> usize {
        self.0 as usize
    }

    pub const fn bit(&self) -> u16 {
        1u16 << self.0
    }

    pub const fn value(&self) -> Value {
        Value::new(self.0 + 1)
    }

    pub const fn label(&self) -> char {
        (b'1' + self.0) as char
    }

    pub const fn highlight(&self) -> char {
        HIGHLIGHT_LABELS[self.usize()]
    }
}

impl TryFrom<char> for Digit {
    type Error = DigitError;

    fn try_from(label: char) -> Result<Self, Self::Error> {
        match label {
            '1'..='9' => Ok(Self(label as u8 - b'1')),
            _ => Err(DigitError::InvalidValue(label.to_string())),
        }
    }
}

impl FromStr for Digit {
    type Err = DigitError;

    fn from_str(label: &str) -> Result<Self, Self::Err> {
        let trimmed = label.trim();
        match trimmed.len() {
            1 => {
                let ch = trimmed.chars().next().unwrap();
                match ch {
                    '1'..='9' => Ok(Self(ch as u8 - b'1')),
                    _ => Err(DigitError::InvalidValue(label.to_string())),
                }
            }
            _ => Err(DigitError::InvalidValue(label.to_string())),
        }
    }
}

impl Add<Digit> for Digit {
    type Output = DigitSet;

    fn add(self, rhs: Digit) -> DigitSet {
        DigitSet::empty() + self + rhs
    }
}

impl Neg for Digit {
    type Output = DigitSet;

    fn neg(self) -> DigitSet {
        DigitSet::full() - self
    }
}

impl fmt::Display for Digit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

pub struct DigitIter(u8);

impl DigitIter {
    pub const fn new() -> Self {
        Self(0)
    }
}

impl Iterator for DigitIter {
    type Item = Digit;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 < 9 {
            let digit = Digit::new(self.0.into());
            self.0 += 1;
            Some(digit)
        } else {
            None
        }
    }
}

impl ExactSizeIterator for DigitIter {
    fn len(&self) -> usize {
        9 - self.0 as usize
    }
}

/// Creates a [`Digit`] from a digit character.
///
/// Compile-time convenience that panics on invalid input.
/// For runtime parsing with error handling, use [`Digit::from_str`] or `"5".parse::<Digit>()`.
///
/// # Examples
///
/// ```
/// use sudoku_rust::digit;
///
/// let d = digit!(5);
/// let d = digit!(9);
/// ```
///
/// # Panics
///
/// Panics if the value is not 1-9. See [`Digit::from_str`] for valid formats.
#[macro_export]
macro_rules! digit {
    ($value:tt) => {
        match stringify!($value).parse::<Digit>() {
            Ok(d) => d,
            Err(e) => panic!("digit![{}]: {}", stringify!($value), e),
        }
    };
}

const HIGHLIGHT_LABELS: [char; 9] = ['❶', '❷', '❸', '❹', '❺', '❻', '❼', '❽', '❾'];
