use std::fmt;

use super::Cell;

/// A rectangle of four cells.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Rectangle {
    top_left: Cell,
    bottom_right: Cell,
}

impl Rectangle {
    pub fn new(top_left: Cell, bottom_right: Cell) -> Rectangle {
        Rectangle {
            top_left,
            bottom_right,
        }
    }

    pub fn from(cells: [Cell; 4]) -> Rectangle {
        let mut sorted = cells;
        sorted.sort();
        Rectangle {
            top_left: sorted[0],
            bottom_right: sorted[3],
        }
    }
}

impl fmt::Debug for Rectangle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Rectangle({} {})", self.top_left, self.bottom_right)
    }
}

impl fmt::Display for Rectangle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "R{}{}C{}{}",
            self.top_left.row().coord(),
            self.bottom_right.row().coord(),
            self.top_left.column().coord(),
            self.bottom_right.column().coord(),
        )
    }
}
