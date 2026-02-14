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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbols::MISSING;

    #[test]
    fn none_and_default_are_missing() {
        let none = Value::none();
        let defaulted = Value::default();

        assert!(none.is_none());
        assert!(defaulted.is_none());
        assert_eq!(Value::NONE, none.value());
        assert_eq!(MISSING, none.label());
    }

    #[test]
    fn new_digit_is_detected() {
        let value = Value::new(5);

        assert!(value.is_digit());
        assert_eq!(Some(Digit::from_ordinal(5)), value.digit());
        assert_eq!(5, value.value());
        assert_eq!('5', value.label());
    }

    #[test]
    fn digit_returns_none_for_missing() {
        let value = Value::none();

        assert_eq!(None, value.digit());
    }

    #[test]
    fn conversions_work() {
        let from_digit: Value = Digit::from_ordinal(9).into();
        let from_u8: Value = 4u8.into();
        let from_char = Value::from('7');
        let from_zero = Value::from('0');
        let from_str = Value::from("8");

        assert_eq!(9, from_digit.value());
        assert_eq!(4, from_u8.value());
        assert_eq!(7, from_char.value());
        assert!(from_zero.is_none());
        assert_eq!(8, from_str.value());
    }

    #[test]
    fn not_operator_matches_missing_state() {
        assert!(!Value::none());
        assert!(!(!Value::new(3)));
    }

    #[test]
    fn formatting_uses_labels() {
        let value = Value::new(2);

        assert_eq!("2", format!("{}", value));
        assert_eq!("2", format!("{:?}", value));
        assert_eq!(MISSING.to_string(), format!("{}", Value::none()));
    }

    #[test]
    fn value_macro_creates_value() {
        let value = crate::value!(3);

        assert_eq!(3, value.value());
    }
}
