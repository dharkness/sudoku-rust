use std::fmt;

#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Coord(u8);

/// Errors that can occur when parsing a coordinate label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoordError {
    InvalidValue(String),
}

impl fmt::Display for CoordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoordError::InvalidValue(label) => {
                write!(f, "invalid coord '{}'", label)
            }
        }
    }
}

impl std::error::Error for CoordError {}

/// Identifies a row, column, or block or a cell in a house.
impl Coord {
    pub const COUNT: u8 = 9;

    pub const fn new(coord: u8) -> Self {
        debug_assert!(coord < 9);
        Self(coord)
    }

    pub const fn from_digit(digit: u8) -> Self {
        debug_assert!(1 <= digit && digit <= 9);
        Self(digit - 1)
    }

    pub const fn from_index(index: u32) -> Self {
        debug_assert!(index < 9);
        Self(index as u8)
    }

    pub const fn u8(&self) -> u8 {
        self.0
    }

    pub const fn usize(&self) -> usize {
        self.0 as usize
    }

    pub const fn bit(&self) -> u16 {
        1 << self.0
    }

    pub const fn label(&self) -> char {
        (b'1' + self.0) as char
    }

    pub const fn min(self, other: Self) -> Self {
        if self.0 <= other.0 {
            self
        } else {
            other
        }
    }

    pub const fn max(self, other: Self) -> Self {
        if self.0 >= other.0 {
            self
        } else {
            other
        }
    }
}

impl TryFrom<char> for Coord {
    type Error = CoordError;

    fn try_from(label: char) -> Result<Self, Self::Error> {
        match label {
            '1'..='9' => Ok(Self(label as u8 - b'1')),
            'A'..='H' => Ok(Self(label as u8 - b'A')),
            'a'..='h' => Ok(Self(label as u8 - b'a')),
            'J' | 'j' => Ok(Self(8)),
            _ => Err(CoordError::InvalidValue(label.to_string())),
        }
    }
}

impl TryFrom<&str> for Coord {
    type Error = CoordError;

    fn try_from(label: &str) -> Result<Self, Self::Error> {
        let trimmed = label.trim();
        match trimmed.len() {
            1 => Coord::try_from(trimmed.chars().next().unwrap()),
            _ => Err(CoordError::InvalidValue(label.to_string())),
        }
    }
}

impl TryFrom<String> for Coord {
    type Error = CoordError;

    fn try_from(label: String) -> Result<Self, Self::Error> {
        Self::try_from(label.as_str())
    }
}

impl From<i32> for Coord {
    fn from(coord: i32) -> Self {
        debug_assert!((1..=9).contains(&coord));
        Self::new(coord as u8 - 1)
    }
}

impl From<u8> for Coord {
    fn from(coord: u8) -> Self {
        Self::new(coord)
    }
}

impl fmt::Display for Coord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Convenience macro to create a `Coord` from a row, column, or block label.
///
/// Supported forms:
/// - coord!(2) -- digit for column or block
/// - coord!(B) -- letter for row
///
/// The label should be in [1-9] or [A-H,J].
#[macro_export]
macro_rules! coord {
    ($label:tt) => {
        match Coord::try_from(stringify!($label)) {
            Ok(coord) => coord,
            Err(e) => panic!("coord![{}]: {}", stringify!($value), e),
        }
    };
}
