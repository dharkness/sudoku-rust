use crate::layout::{Cell, House, Known, Rectangle};
use std::fmt;

/// Tracks an error encountered while solving a cell or removing a candidate.
#[derive(Clone, Copy, Debug)]
pub enum Error {
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
            Error::UnsolvableCell(cell) => write!(f, "{} has no candidates", cell),
            Error::UnsolvableHouse(house, known) => {
                write!(f, "{} has no candidate cells in {}", known, house)
            }
            Error::DeadlyRectangle(rectangle) => write!(f, "{} form a deadly rectangle", rectangle),
        }
    }
}
