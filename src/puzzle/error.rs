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
