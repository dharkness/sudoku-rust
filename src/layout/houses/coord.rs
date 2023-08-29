use std::fmt;

#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Coord(u8);

/// Identifies a cell in a house.
impl Coord {
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

impl From<i32> for Coord {
    fn from(coord: i32) -> Self {
        debug_assert!(coord >= 0);
        Self::new(coord as u8)
    }
}

impl From<u8> for Coord {
    fn from(coord: u8) -> Self {
        Self::new(coord)
    }
}

impl From<char> for Coord {
    fn from(coord: char) -> Self {
        Self::new(coord as u8 - b'1')
    }
}

impl From<&str> for Coord {
    fn from(label: &str) -> Self {
        Coord::from(label.chars().next().unwrap())
    }
}

impl From<usize> for Coord {
    fn from(coord: usize) -> Self {
        Self::new(coord as u8)
    }
}

impl fmt::Display for Coord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[allow(unused_macros)]
macro_rules! coord {
    ($c:expr) => {
        Coord::new($c as u8 - 1)
    };
}

#[allow(unused_imports)]
pub(crate) use coord;
