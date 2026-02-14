use std::fmt;
use std::str::FromStr;

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

    pub const fn new(index: u8) -> Self {
        debug_assert!(index < 9);
        Self(index)
    }

    pub const fn from_ordinal(digit: u8) -> Self {
        debug_assert!(1 <= digit && digit <= 9);
        Self(digit - 1)
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

    pub const fn row_label(&self) -> char {
        if self.0 < 8 {
            (b'A' + self.0) as char
        } else {
            'J'
        }
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

impl FromStr for Coord {
    type Err = CoordError;

    fn from_str(label: &str) -> Result<Self, Self::Err> {
        let trimmed = label.trim();
        match trimmed.len() {
            1 => {
                let ch = trimmed.chars().next().unwrap();
                match ch {
                    '1'..='9' => Ok(Self(ch as u8 - b'1')),
                    'A'..='H' => Ok(Self(ch as u8 - b'A')),
                    'a'..='h' => Ok(Self(ch as u8 - b'a')),
                    'J' | 'j' => Ok(Self(8)),
                    _ => Err(CoordError::InvalidValue(label.to_string())),
                }
            }
            _ => Err(CoordError::InvalidValue(label.to_string())),
        }
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
        match stringify!($label).parse::<Coord>() {
            Ok(coord) => coord,
            Err(e) => panic!("coord![{}]: {}", stringify!($label), e),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_and_accessors_work() {
        let coord = Coord::new(4);

        assert_eq!(4, coord.u8());
        assert_eq!(4usize, coord.usize());
        assert_eq!(1u16 << 4, coord.bit());
        assert_eq!('5', coord.label());
        assert_eq!('E', coord.row_label());
    }

    #[test]
    fn from_ordinal_maps_digit_to_index() {
        let coord = Coord::from_ordinal(1);

        assert_eq!(0, coord.u8());
        assert_eq!('1', coord.label());
    }

    #[test]
    fn row_label_uses_j_for_last_row() {
        assert_eq!('H', Coord::new(7).row_label());
        assert_eq!('J', Coord::new(8).row_label());
    }

    #[test]
    fn min_and_max_select_bounds() {
        let a = Coord::new(2);
        let b = Coord::new(6);

        assert_eq!(a, a.min(b));
        assert_eq!(b, a.max(b));
    }

    #[test]
    fn try_from_and_parse_handle_valid_and_invalid() {
        assert_eq!(Coord::new(0), Coord::try_from('1').unwrap());
        assert_eq!(Coord::new(0), Coord::try_from('A').unwrap());
        assert_eq!(Coord::new(7), Coord::try_from('h').unwrap());
        assert_eq!(Coord::new(8), Coord::try_from('J').unwrap());
        assert_eq!(Coord::new(1), " b ".parse::<Coord>().unwrap());

        let err = Coord::try_from('I').unwrap_err();
        assert_eq!(CoordError::InvalidValue("I".to_string()), err);
        assert_eq!("invalid coord 'I'", err.to_string());
        assert!("12".parse::<Coord>().is_err());
        assert!("0".parse::<Coord>().is_err());
    }

    #[test]
    fn conversions_and_display() {
        let coord: Coord = 6u8.into();

        assert_eq!('7', coord.label());
        assert_eq!("7", format!("{}", coord));
    }

    #[test]
    fn coord_macro_creates_coords() {
        let digit = crate::coord!(2);
        let row = crate::coord!(B);

        assert_eq!('2', digit.label());
        assert_eq!('B', row.row_label());
    }
}
