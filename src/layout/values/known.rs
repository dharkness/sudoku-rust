use std::fmt;
use std::ops::{Add, Neg};
use std::str::FromStr;

use super::{KnownSet, Value};

/// Holds one of the possible known values.
#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Known(u8);

/// Errors that can occur when parsing a known value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KnownError {
    InvalidValue(String),
}

impl fmt::Display for KnownError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KnownError::InvalidValue(label) => {
                write!(f, "invalid known '{}'", label)
            }
        }
    }
}

impl std::error::Error for KnownError {}

impl Known {
    pub const COUNT: u8 = 9;

    pub fn iter() -> KnownIter {
        KnownIter::new()
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

impl TryFrom<char> for Known {
    type Error = KnownError;

    fn try_from(label: char) -> Result<Self, Self::Error> {
        match label {
            '1'..='9' => Ok(Self(label as u8 - b'1')),
            _ => Err(KnownError::InvalidValue(label.to_string())),
        }
    }
}

impl FromStr for Known {
    type Err = KnownError;

    fn from_str(label: &str) -> Result<Self, Self::Err> {
        let trimmed = label.trim();
        match trimmed.len() {
            1 => {
                let ch = trimmed.chars().next().unwrap();
                match ch {
                    '1'..='9' => Ok(Self(ch as u8 - b'1')),
                    _ => Err(KnownError::InvalidValue(label.to_string())),
                }
            }
            _ => Err(KnownError::InvalidValue(label.to_string())),
        }
    }
}

impl Add<Known> for Known {
    type Output = KnownSet;

    fn add(self, rhs: Known) -> KnownSet {
        KnownSet::empty() + self + rhs
    }
}

impl Neg for Known {
    type Output = KnownSet;

    fn neg(self) -> KnownSet {
        KnownSet::full() - self
    }
}

impl fmt::Display for Known {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

pub struct KnownIter(u8);

impl KnownIter {
    pub const fn new() -> Self {
        Self(0)
    }
}

impl Iterator for KnownIter {
    type Item = Known;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 < 9 {
            let known = Known::new(self.0.into());
            self.0 += 1;
            Some(known)
        } else {
            None
        }
    }
}

impl ExactSizeIterator for KnownIter {
    fn len(&self) -> usize {
        9 - self.0 as usize
    }
}

/// Creates a [`Known`] from a digit character.
///
/// Compile-time convenience that panics on invalid input.
/// For runtime parsing with error handling, use [`Known::from_str`] or `"5".parse::<Known>()`.
///
/// # Examples
///
/// ```
/// use sudoku_rust::known;
///
/// let k = known!(5);
/// let k = known!(9);
/// ```
///
/// # Panics
///
/// Panics if the value is not 1-9. See [`Known::from_str`] for valid formats.
#[macro_export]
macro_rules! known {
    ($value:tt) => {
        match stringify!($value).parse::<Known>() {
            Ok(k) => k,
            Err(e) => panic!("known![{}]: {}", stringify!($value), e),
        }
    };
}

const HIGHLIGHT_LABELS: [char; 9] = ['❶', '❷', '❸', '❹', '❺', '❻', '❼', '❽', '❾'];
