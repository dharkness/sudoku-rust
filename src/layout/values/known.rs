use std::fmt;
use std::ops::{Add, Neg};

use super::{KnownSet, Value};

/// Holds one of the possible known values.
#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Known(u8);

impl Known {
    pub const ALL: [Known; 9] = [
        known!(1),
        known!(2),
        known!(3),
        known!(4),
        known!(5),
        known!(6),
        known!(7),
        known!(8),
        known!(9),
    ];

    pub const fn new(value: u8) -> Self {
        debug_assert!(1 <= value && value <= 9);
        Self(value - 1)
    }

    pub const fn from_index(index: u32) -> Self {
        debug_assert!(index < 9);
        Self(index as u8)
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
}

impl From<u8> for Known {
    fn from(index: u8) -> Self {
        assert!(index < 9);
        Known::new(index + 1)
    }
}

impl From<char> for Known {
    fn from(label: char) -> Self {
        if !('1'..='9').contains(&label) {
            panic!("Invalid known \"{}\"", label);
        }
        Known::new(label as u8 - b'0')
    }
}

impl From<&str> for Known {
    fn from(label: &str) -> Self {
        Known::from(label.chars().next().unwrap())
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

#[allow(unused_macros)]
macro_rules! known {
    ($k:expr) => {
        Known::new($k as u8)
    };
}

#[allow(unused_imports)]
pub(crate) use known;
