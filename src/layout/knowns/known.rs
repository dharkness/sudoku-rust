use std::ops::{Deref, DerefMut};

use super::Set;

/// Specifies a single known value using its index and bit.
#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Known {
    index: u16,
    bit: u16,
}

impl Known {
    pub const UNKNOWN: u8 = 0;
    pub const ALL: [Known; 9] = [
        Known::new(0),
        Known::new(1),
        Known::new(2),
        Known::new(3),
        Known::new(4),
        Known::new(5),
        Known::new(6),
        Known::new(7),
        Known::new(8),
    ];

    pub const fn new(index: u16) -> Self {
        debug_assert!(index < 9);
        Self {
            index,
            bit: 1 << index,
        }
    }

    pub const fn index(&self) -> u16 {
        self.index
    }

    pub const fn usize(&self) -> usize {
        self.index as usize
    }

    pub const fn bit(&self) -> u16 {
        self.bit
    }

    pub const fn value(&self) -> u8 {
        (self.index + 1) as u8
    }

    pub const fn label(&self) -> &'static str {
        LABELS[self.index as usize]
    }
}

impl From<u16> for Known {
    fn from(index: u16) -> Self {
        assert!(index < 9);
        Known::ALL[index as usize]
    }
}

impl From<Known> for usize {
    fn from(known: Known) -> Self {
        known.index as usize
    }
}

impl From<char> for Known {
    fn from(label: char) -> Self {
        Known::ALL[(label as u8 - b'1') as usize]
    }
}

impl From<&str> for Known {
    fn from(label: &str) -> Self {
        Known::ALL[(label.chars().next().unwrap() as u8 - b'1') as usize]
    }
}

impl ToString for Known {
    fn to_string(&self) -> String {
        self.label().to_string()
    }
}

const LABELS: [&str; 9] = ["1", "2", "3", "4", "5", "6", "7", "8", "9"];
