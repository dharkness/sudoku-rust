use std::fmt;
use std::ops::Not;

use crate::symbols::MISSING;

use super::Digit;

/// Holds the value stored in a cell, either unsolved or one of the nine digits.
#[derive(Clone, Copy, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Value(u8);

impl Value {
    pub const NONE: u8 = 0;

    pub const fn none() -> Self {
        Self(Self::NONE)
    }

    pub const fn new(value: u8) -> Self {
        debug_assert!(value <= 9);
        Self(value)
    }

    pub const fn is_none(&self) -> bool {
        self.0 == Self::NONE
    }

    pub const fn is_digit(&self) -> bool {
        self.0 != Self::NONE
    }

    pub const fn digit(&self) -> Option<Digit> {
        if self.is_digit() {
            Some(Digit::from_ordinal(self.0))
        } else {
            None
        }
    }

    pub const fn value(&self) -> u8 {
        self.0
    }

    pub const fn label(&self) -> char {
        if self.is_none() {
            MISSING
        } else {
            (b'0' + self.0) as char
        }
    }
}

impl From<Digit> for Value {
    fn from(digit: Digit) -> Self {
        digit.value()
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
            Value::none();
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
        self.is_none()
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

#[macro_export]
macro_rules! value {
    ($k:expr) => {
        Value::new($k as u8)
    };
}
