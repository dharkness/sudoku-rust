use std::fmt;
use std::hash::{Hash, Hasher};

use super::{Cell, CellSet};

/// A rectangle of four cells.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Rectangle {
    top_left: Cell,
    top_right: Cell,
    bottom_left: Cell,
    bottom_right: Cell,
    cells: CellSet,
    block_count: usize,
}

impl Rectangle {
    pub const fn new(top_left: Cell, bottom_right: Cell) -> Rectangle {
        let top_right = Cell::from_coords(top_left.row_coord(), bottom_right.column_coord());
        let bottom_left = Cell::from_coords(bottom_right.row_coord(), top_left.column_coord());
        let cells = CellSet::of(&[top_left, top_right, bottom_left, bottom_right]);

        let tl_block = top_left.block_coord().usize();
        let br_block = bottom_right.block_coord().usize();
        let block_count = if tl_block == br_block {
            1
        } else if tl_block % 3 == br_block % 3 || tl_block / 3 == br_block / 3 {
            2
        } else {
            4
        };

        Rectangle {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
            cells,
            block_count,
        }
    }

    pub fn from(c1: Cell, c2: Cell, c3: Cell, c4: Cell) -> Rectangle {
        Rectangle::new(c1.min(c2).min(c3).min(c4), c1.max(c2).max(c3).max(c4))
    }

    pub const fn top_left(&self) -> Cell {
        self.top_left
    }

    pub const fn top_right(&self) -> Cell {
        self.top_right
    }

    pub const fn bottom_left(&self) -> Cell {
        self.bottom_left
    }

    pub const fn bottom_right(&self) -> Cell {
        self.bottom_right
    }

    pub const fn cells(&self) -> CellSet {
        self.cells
    }

    pub const fn block_count(&self) -> usize {
        self.block_count
    }
}

impl Hash for Rectangle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.top_left.hash(state);
        self.bottom_right.hash(state);
    }
}

impl TryFrom<Vec<Cell>> for Rectangle {
    type Error = ();

    fn try_from(cells: Vec<Cell>) -> Result<Rectangle, ()> {
        Rectangle::try_from(CellSet::from_iter(cells))
    }
}

impl TryFrom<CellSet> for Rectangle {
    type Error = ();

    fn try_from(cells: CellSet) -> Result<Rectangle, ()> {
        if cells.size() < 2 || 4 < cells.size() {
            return Err(());
        }

        let rows = cells.rows();
        let columns = cells.columns();

        if rows.size() != 2 || columns.size() != 2 {
            return Err(());
        }

        let (top, bottom) = rows.as_pair().unwrap();
        let (left, right) = columns.as_pair().unwrap();

        Ok(Rectangle::new(
            Cell::from_coords(top.coord(), left.coord()),
            Cell::from_coords(bottom.coord(), right.coord()),
        ))
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
