use std::fmt;

/// Specifies a single known value using its zero-based index.
#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Known(u8);

impl Known {
    pub const UNKNOWN: u8 = 0;
    pub const ALL: [Known; 9] = [
        Known::new(1),
        Known::new(2),
        Known::new(3),
        Known::new(4),
        Known::new(5),
        Known::new(6),
        Known::new(7),
        Known::new(8),
        Known::new(9),
    ];

    pub const fn new(value: u8) -> Self {
        debug_assert!(1 <= value && value <= 9);
        Self(value - 1)
    }

    pub const fn usize(&self) -> usize {
        self.0 as usize
    }

    pub const fn bit(&self) -> u16 {
        1u16 << self.0
    }

    pub const fn value(&self) -> u8 {
        self.0 + 1
    }

    pub const fn label(&self) -> &'static str {
        LABELS[self.usize()]
    }
}

impl From<u8> for Known {
    fn from(index: u8) -> Self {
        assert!(index < 9);
        Known::ALL[index as usize]
    }
}

impl From<char> for Known {
    fn from(label: char) -> Self {
        Known::ALL[(label as u8 - b'1') as usize]
    }
}

impl From<&str> for Known {
    fn from(label: &str) -> Self {
        Known::from(label.chars().next().unwrap())
    }
}

impl fmt::Display for Known {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())?;
        Ok(())
    }
}

const LABELS: [&str; 9] = ["1", "2", "3", "4", "5", "6", "7", "8", "9"];
