use std::fmt;

use crate::layout::{Cell, House, Known, Rectangle};

/// Tracks an error encountered while solving a cell or removing a candidate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// Cannot solve a cell to a non-candidate.
    NotCandidate(Cell, Known),
    /// Cannot solve a cell that is already solved with a different known.
    AlreadySolved(Cell, Known, Known),

    /// The unsolved cell has no more candidates remaining.
    UnsolvableCell(Cell),
    /// An unsolved value has no more candidate cells in the house.
    UnsolvableHouse(House, Known),

    /// Four cells in two boxes form a deadly rectangle.
    DeadlyRectangle(Rectangle),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::NotCandidate(cell, known) => {
                write!(f, "{} cannot be solved with {}", cell, known)
            }
            Error::AlreadySolved(cell, known, current) => write!(
                f,
                "{} cannot be changed from {} to {}",
                cell, current, known
            ),

            Error::UnsolvableCell(cell) => write!(f, "{} has no candidates", cell),
            Error::UnsolvableHouse(house, known) => {
                write!(f, "{} has no candidate cells for {}", house, known)
            }

            Error::DeadlyRectangle(rectangle) => write!(f, "{} form a deadly rectangle", rectangle),
        }
    }
}
