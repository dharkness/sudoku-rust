use std::ops::{Deref, DerefMut};

use super::{Cell, CellSet};

/// Identifies a cell within a row, column or block.
pub type Coord = u8;

/// Identifies one of the nine rows from top to bottom by its coordinate.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Row(Coord);

impl Row {
    pub const fn new(row: Coord) -> Self {
        debug_assert!(row <= 8);
        Self(row)
    }

    pub const fn coord(&self) -> Coord {
        self.0
    }

    pub const fn usize(&self) -> usize {
        self.0 as usize
    }

    pub const fn cell(&self, column: Column) -> Cell {
        Cell::new((9 * self.0 + column.coord()) as u32)
    }

    pub fn cells(&self) -> CellSet {
        let mut set = CellSet::empty();
        (0..9).for_each(|i| set += self.cell(Column::new(i)));
        set
    }
}

impl Deref for Row {
    type Target = Coord;

    fn deref(&self) -> &Coord {
        &self.0
    }
}

impl DerefMut for Row {
    fn deref_mut(&mut self) -> &mut Coord {
        &mut self.0
    }
}

/// Identifies one of the nine columns from left to right by its coordinate.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Column(u8);

impl Column {
    pub const fn new(column: Coord) -> Self {
        debug_assert!(column <= 8);
        Self(column)
    }

    pub const fn coord(&self) -> Coord {
        self.0
    }

    pub const fn usize(&self) -> usize {
        self.0 as usize
    }

    pub const fn cell(&self, row: Row) -> Cell {
        Cell::new((9 * row.coord() + self.0) as u32)
    }

    pub fn cells(&self) -> CellSet {
        let mut set = CellSet::empty();
        (0..9).for_each(|i| set += self.cell(Row::new(i)));
        set
    }
}

impl Deref for Column {
    type Target = Coord;

    fn deref(&self) -> &Coord {
        &self.0
    }
}

impl DerefMut for Column {
    fn deref_mut(&mut self) -> &mut Coord {
        &mut self.0
    }
}

/// Identifies one of the nine blocks from left to right and top to bottom by its coordinate.
/// It is called a Box in output, but Rust defines Box.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Block(u8);

impl Block {
    pub const fn new(block: Coord) -> Self {
        debug_assert!(block <= 8);
        Self(block)
    }

    pub const fn coord(&self) -> Coord {
        self.0
    }

    pub const fn usize(&self) -> usize {
        self.0 as usize
    }

    pub const fn cell(&self, coord: Coord) -> Cell {
        debug_assert!(coord <= 8);
        Cell::new(((self.0 / 3) * 27 + (self.0 % 3) * 3 + (coord / 3) * 9 + (coord % 3)) as u32)
    }

    pub fn cells(&self) -> CellSet {
        let mut set = CellSet::empty();
        (0..9).for_each(|i| set += self.cell(i));
        set
    }
}

impl Deref for Block {
    type Target = Coord;

    fn deref(&self) -> &Coord {
        &self.0
    }
}

impl DerefMut for Block {
    fn deref_mut(&mut self) -> &mut Coord {
        &mut self.0
    }
}
