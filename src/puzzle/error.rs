use std::fmt;

use crate::layout::{Cell, Digit, House, Rectangle};

/// Tracks an error encountered while solving a cell or removing a candidate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// Cannot solve a cell to a non-candidate.
    NotCandidate(Cell, Digit),
    /// Cannot solve a cell that is already solved with a different digit.
    AlreadySolved(Cell, Digit, Digit),

    /// The unsolved cell has no more candidates remaining.
    UnsolvableCell(Cell),
    /// An unsolved value has no more candidate cells in the house.
    UnsolvableHouse(House, Digit),

    /// Four cells in two boxes form a deadly rectangle.
    DeadlyRectangle(Rectangle),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::NotCandidate(cell, digit) => {
                write!(f, "{} cannot be solved with {}", cell, digit)
            }
            Error::AlreadySolved(cell, digit, current) => write!(
                f,
                "{} cannot be changed from {} to {}",
                cell, current, digit
            ),

            Error::UnsolvableCell(cell) => write!(f, "{} has no candidates", cell),
            Error::UnsolvableHouse(house, digit) => {
                write!(f, "{} has no candidate cells for {}", house, digit)
            }

            Error::DeadlyRectangle(rectangle) => {
                write!(f, "{} forms a deadly rectangle", rectangle)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Coord, Rectangle};
    use crate::*;

    #[test]
    fn display_not_candidate() {
        let error = Error::NotCandidate(cell!(A1), digit!(3));

        assert_eq!("A1 cannot be solved with 3", error.to_string());
    }

    #[test]
    fn display_already_solved() {
        let error = Error::AlreadySolved(cell!(A1), digit!(2), digit!(1));

        assert_eq!("A1 cannot be changed from 1 to 2", error.to_string());
    }

    #[test]
    fn display_unsolvable_cell() {
        let error = Error::UnsolvableCell(cell!(B2));

        assert_eq!("B2 has no candidates", error.to_string());
    }

    #[test]
    fn display_unsolvable_house() {
        let error = Error::UnsolvableHouse(House::row(Coord::new(0)), digit!(4));

        assert_eq!("Row A has no candidate cells for 4", error.to_string());
    }

    #[test]
    fn display_deadly_rectangle() {
        let rectangle = Rectangle::new(cell!(A1), cell!(B2));
        let error = Error::DeadlyRectangle(rectangle);

        assert_eq!("R12C12 forms a deadly rectangle", error.to_string());
    }
}
