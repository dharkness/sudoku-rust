use std::fmt;
use std::ops::Not;

use crate::symbols::MISSING;

use super::Known;

/// Holds the value stored in a cell, either unknown or one of the nine digits.
#[derive(Clone, Copy, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Value(u8);

impl Value {
    pub const UNKNOWN: u8 = 0;

    pub const fn unknown() -> Self {
        Self(Self::UNKNOWN)
    }

    pub const fn new(value: u8) -> Self {
        debug_assert!(value <= 9);
        Self(value)
    }

    pub const fn is_unknown(&self) -> bool {
        self.0 == Self::UNKNOWN
    }

    pub const fn is_known(&self) -> bool {
        self.0 != Self::UNKNOWN
    }

    pub const fn known(&self) -> Option<Known> {
        if self.is_known() {
            Some(Known::new(self.0))
        } else {
            None
        }
    }

    pub const fn value(&self) -> u8 {
        self.0
    }

    pub const fn label(&self) -> char {
        if self.is_unknown() {
            MISSING
        } else {
            (b'0' + self.0) as char
        }
    }
}

impl From<Known> for Value {
    fn from(known: Known) -> Self {
        known.value()
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Value::new(value)
    }
}

impl From<char> for Value {
    fn from(label: char) -> Self {
        if !('1'..='9').contains(&label) {
            Value::unknown();
        }
        Value::new(label as u8 - b'0')
    }
}

impl From<&str> for Value {
    fn from(label: &str) -> Self {
        Value::from(label.chars().next().unwrap())
    }
}

impl Not for Value {
    type Output = bool;

    fn not(self) -> bool {
        self.is_unknown()
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[allow(unused_macros)]
macro_rules! value {
    ($k:expr) => {
        Value::new($k as u8)
    };
}

#[allow(unused_imports)]
pub(crate) use value;
