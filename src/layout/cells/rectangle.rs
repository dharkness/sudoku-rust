use std::fmt;
use std::hash::{Hash, Hasher};
use std::iter::FusedIterator;

use crate::layout::{Coord, House};

use super::{Cell, CellSet};

/// A rectangle of four cells.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Rectangle {
    pub top_left: Cell,
    pub top_right: Cell,
    pub bottom_left: Cell,
    pub bottom_right: Cell,
    pub cells: CellSet,
    pub block_count: usize,
}

impl Rectangle {
    pub fn iter() -> RectangleIter {
        RectangleIter::default()
    }

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

    /// Returns a flipped copy to move the origin to the top-left cell.
    pub fn with_origin(self, origin: Cell) -> Rectangle {
        if origin == self.bottom_right {
            Rectangle {
                top_left: self.bottom_right,
                top_right: self.bottom_left,
                bottom_left: self.top_right,
                bottom_right: self.top_left,
                cells: self.cells,
                block_count: self.block_count,
            }
        } else if origin == self.top_right {
            Rectangle {
                top_left: self.top_right,
                top_right: self.top_left,
                bottom_left: self.bottom_right,
                bottom_right: self.bottom_left,
                cells: self.cells,
                block_count: self.block_count,
            }
        } else if origin == self.bottom_left {
            Rectangle {
                top_left: self.bottom_left,
                top_right: self.bottom_right,
                bottom_left: self.top_left,
                bottom_right: self.top_right,
                cells: self.cells,
                block_count: self.block_count,
            }
        } else {
            self
        }
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
        if cells.len() < 2 || 4 < cells.len() {
            return Err(());
        }

        let rows = cells.rows();
        let columns = cells.columns();

        if rows.len() != 2 || columns.len() != 2 {
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

/// Iterates through all unique two-block rectangles.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RectangleIter {
    horiz_vert: usize,
    block: usize,
    cell: usize,
}

impl Iterator for RectangleIter {
    type Item = Rectangle;

    fn next(&mut self) -> Option<Rectangle> {
        if self.horiz_vert == 2 {
            return None;
        }

        let (from, to) = BLOCKS[self.horiz_vert][self.block];
        let ((tl, _bl), (_tr, br)) = CELL_COORDS[self.horiz_vert][self.cell];

        let rect = Rectangle::new(from.cell(tl), to.cell(br));

        self.cell += 1;
        if self.cell == 27 {
            self.cell = 0;
            self.block += 1;
            if self.block == 9 {
                self.block = 0;
                self.horiz_vert += 1;
            }
        }

        Some(rect)
    }
}

impl FusedIterator for RectangleIter {}

/// A pair of coordinates, either two different boxes or two different cells in the same box.
type IndexPair = (u8, u8);

/// A pair of cell coordinates in a box.
type CoordPair = (Coord, Coord);

/// The block pairs (from, to) to check for deadly rectangles.
/// All possible rectangles between the two blocks are checked
/// using the coordinates below.
const BLOCKS: [[(House, House); 9]; 2] = {
    #[rustfmt::skip]
    const BLOCKS: [[IndexPair; 9]; 2] = [
        // horizontal
        [
            (0, 1), (0, 2), (1, 2),
            (3, 4), (3, 5), (4, 5),
            (6, 7), (6, 8), (7, 8),
        ],
        // vertical
        [
            (0, 3), (0, 6), (3, 6),
            (1, 4), (1, 7), (4, 7),
            (2, 5), (2, 8), (5, 8),
        ],
    ];
    const DEFAULT: House = House::block(Coord::new(0));

    let mut blocks: [[(House, House); 9]; 2] = [[(DEFAULT, DEFAULT); 9]; 2];
    let mut horiz_vert = 0;

    while horiz_vert < 2 {
        let mut i = 0;
        while i < 9 {
            let (f, t) = BLOCKS[horiz_vert][i];
            blocks[horiz_vert][i] = (House::block(Coord::new(f)), House::block(Coord::new(t)));
            i += 1;
        }
        horiz_vert += 1;
    }

    blocks
};

/// Cell coordinates (top-left, bottom-right) for each rectangle.
/// each in a different block in the pairs above.
const CELL_COORDS: [[(CoordPair, CoordPair); 27]; 2] = {
    #[rustfmt::skip]
    const COORDS: [[(IndexPair, IndexPair); 27]; 2] = [
        // horizontal
        [
            ((0, 3), (0, 3)), ((0, 3), (1, 4)), ((0, 3), (2, 5)),
            ((0, 6), (0, 6)), ((0, 6), (1, 7)), ((0, 6), (2, 8)),
            ((3, 6), (3, 6)), ((3, 6), (4, 7)), ((3, 6), (5, 8)),

            ((1, 4), (0, 3)), ((1, 4), (1, 4)), ((1, 4), (2, 5)),
            ((1, 7), (0, 6)), ((1, 7), (1, 7)), ((1, 7), (2, 8)),
            ((4, 7), (3, 6)), ((4, 7), (4, 7)), ((4, 7), (5, 8)),

            ((2, 5), (0, 3)), ((2, 5), (1, 4)), ((2, 5), (2, 5)),
            ((2, 8), (0, 6)), ((2, 8), (1, 7)), ((2, 8), (2, 8)),
            ((5, 8), (3, 6)), ((5, 8), (4, 7)), ((5, 8), (5, 8)),
        ],
        // vertical
        [
            ((0, 1), (0, 1)), ((0, 1), (3, 4)), ((0, 1), (6, 7)),
            ((0, 2), (0, 2)), ((0, 2), (3, 5)), ((0, 2), (6, 8)),
            ((1, 2), (1, 2)), ((1, 2), (4, 5)), ((1, 2), (7, 8)),

            ((3, 4), (0, 1)), ((3, 4), (3, 4)), ((3, 4), (6, 7)),
            ((3, 5), (0, 2)), ((3, 5), (3, 5)), ((3, 5), (6, 8)),
            ((4, 5), (1, 2)), ((4, 5), (4, 5)), ((4, 5), (7, 8)),

            ((6, 7), (0, 1)), ((6, 7), (3, 4)), ((6, 7), (6, 7)),
            ((6, 8), (0, 2)), ((6, 8), (3, 5)), ((6, 8), (6, 8)),
            ((7, 8), (1, 2)), ((7, 8), (4, 5)), ((7, 8), (7, 8)),
        ],
    ];
    const DEFAULT_COORD: Coord = Coord::new(0);
    const DEFAULT: (CoordPair, CoordPair) = (
        (DEFAULT_COORD, DEFAULT_COORD),
        (DEFAULT_COORD, DEFAULT_COORD),
    );

    let mut coords: [[(CoordPair, CoordPair); 27]; 2] = [[DEFAULT; 27]; 2];
    let mut horiz_vert = 0;

    while horiz_vert < 2 {
        let mut i = 0;
        while i < 27 {
            let ((tl, bl), (tr, br)) = COORDS[horiz_vert][i];
            coords[horiz_vert][i] = (
                (Coord::new(tl), Coord::new(bl)),
                (Coord::new(tr), Coord::new(br)),
            );
            i += 1;
        }
        horiz_vert += 1;
    }

    coords
};
