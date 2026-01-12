use std::fmt;
use std::ops::{Add, Neg};

use crate::layout::{Coord, House, Shape};

use super::{Bit, CellSet};

/// Specifies a single cell by its index from left to right and top to bottom.
#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Cell(u8);

/// Errors that can occur when parsing a cell label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CellError {
    WrongLength(String),
    InvalidRow { label: String, row: char },
    InvalidColumn { label: String, column: char },
}

impl fmt::Display for CellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CellError::WrongLength(label) => {
                write!(f, "wrong length {} for cell \"{}\"", label.len(), label)
            }
            CellError::InvalidRow { label, row } => {
                write!(f, "invalid row '{}' for cell \"{}\"", row, label)
            }
            CellError::InvalidColumn { label, column } => {
                write!(f, "invalid column '{}' for cell \"{}\"", column, label)
            }
        }
    }
}

impl std::error::Error for CellError {}

impl Cell {
    pub const COUNT: u8 = 81;

    pub fn iter() -> CellIter {
        CellIter::new()
    }

    pub const fn new(index: u8) -> Self {
        debug_assert!(index < Cell::COUNT);
        Self(index)
    }

    pub const fn from_coords(row: Coord, column: Coord) -> Self {
        Self::new(row.u8() * 9 + column.u8())
    }

    pub const fn from_row_column(row: House, column: House) -> Self {
        Self::from_coords(row.coord(), column.coord())
    }

    pub const fn index(&self) -> u8 {
        self.0
    }

    pub const fn usize(&self) -> usize {
        self.0 as usize
    }

    pub const fn bit(&self) -> Bit {
        Bit::new(1 << self.0)
    }

    pub const fn houses(&self) -> [House; 3] {
        [self.row(), self.column(), self.block()]
    }

    pub const fn house(&self, shape: Shape) -> House {
        match shape {
            Shape::Row => self.row(),
            Shape::Column => self.column(),
            Shape::Block => self.block(),
        }
    }

    pub const fn row(&self) -> House {
        House::row(self.row_coord())
    }

    pub const fn row_coord(&self) -> Coord {
        Coord::new(self.0 / 9)
    }

    pub const fn column(&self) -> House {
        House::column(self.column_coord())
    }

    pub const fn column_coord(&self) -> Coord {
        Coord::new(self.0 % 9)
    }

    pub const fn block(&self) -> House {
        House::block(self.block_coord())
    }

    pub const fn block_coord(&self) -> Coord {
        Coord::new((self.0 / 9 / 3) * 3 + (self.0 % 9 / 3))
    }

    pub const fn coord_in_row(&self) -> Coord {
        self.column_coord()
    }

    pub const fn coord_in_column(&self) -> Coord {
        self.row_coord()
    }

    pub const fn coord_in_block(&self) -> Coord {
        COORDS_IN_HOUSES[self.usize()][Shape::Block.usize()]
    }

    pub fn common_row_or_column(&self, peer: Cell) -> Option<House> {
        if self.row() == peer.row() {
            Some(self.row())
        } else if self.column() == peer.column() {
            Some(self.column())
        } else {
            None
        }
    }

    pub fn common_houses(&self, peer: Cell) -> Vec<House> {
        [self.row(), self.column(), self.block()]
            .iter()
            .copied()
            .filter(|house| house.has(peer))
            .collect::<Vec<_>>()
    }

    pub const fn peers(&self) -> CellSet {
        PEERS[self.usize()]
    }

    pub const fn sees(&self, other: Cell) -> bool {
        PEERS[self.usize()].has(other)
    }

    pub const fn label(&self) -> &'static str {
        debug_assert!(self.0 < Cell::COUNT);
        LABELS[self.0 as usize]
    }

    pub fn labels(cells: &Vec<Cell>) -> String {
        let mut labels = String::new();
        labels.push('(');
        for cell in cells {
            labels.push(' ');
            labels.push_str(cell.label());
        }
        labels.push_str(" )");
        labels
    }
}

impl TryFrom<&str> for Cell {
    type Error = CellError;

    fn try_from(label: &str) -> Result<Self, Self::Error> {
        let trimmed = label.trim();
        if trimmed.len() != 2 {
            return Err(CellError::WrongLength(trimmed.to_string()));
        }

        let mut chars = trimmed.chars();
        let row_char = chars.next().unwrap();
        let col_char = chars.next().unwrap();

        let row = Coord::try_from(row_char).map_err(|_| CellError::InvalidRow {
            label: trimmed.to_string(),
            row: row_char,
        })?;

        let col = Coord::try_from(col_char).map_err(|_| CellError::InvalidColumn {
            label: trimmed.to_string(),
            column: col_char,
        })?;

        Ok(Self::from_coords(row, col))
    }
}

impl TryFrom<String> for Cell {
    type Error = CellError;

    fn try_from(label: String) -> Result<Self, Self::Error> {
        Self::try_from(label.as_str())
    }
}

impl Add<Cell> for Cell {
    type Output = CellSet;

    fn add(self, rhs: Cell) -> CellSet {
        CellSet::empty() + self + rhs
    }
}

impl Neg for Cell {
    type Output = CellSet;

    fn neg(self) -> CellSet {
        CellSet::full() - self
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())?;
        Ok(())
    }
}

pub struct CellIter(u8);

impl CellIter {
    pub const fn new() -> Self {
        Self(0)
    }
}

impl Iterator for CellIter {
    type Item = Cell;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 < Cell::COUNT {
            let cell = Cell::new(self.0);
            self.0 += 1;
            Some(cell)
        } else {
            None
        }
    }
}

impl ExactSizeIterator for CellIter {
    fn len(&self) -> usize {
        (Cell::COUNT - self.0) as usize
    }
}

/// Creates a [`Cell`] from a case-insensitive cell label.
///
/// This macro is intended for use in tests and compile-time constants.
/// For runtime parsing, use [`Cell::try_from`] instead.
///
/// # Examples
///
/// ```
/// # use sudoku_rust::{Cell, cell};
/// let c = cell!(A1);
/// assert_eq!(0, c.index());
/// ```
///
/// # Panics
///
/// Panics if the cell label is invalid.
///
/// See [`Cell::try_from`] for valid label format.
#[macro_export]
macro_rules! cell {
    ($value:tt) => {
        match Cell::try_from(stringify!($value)) {
            Ok(k) => k,
            Err(e) => panic!("cell![{}]: {}", stringify!($value), e),
        }
    };
}

/// Every cell's label.
#[rustfmt::skip]
const LABELS: [&str; 81] = [
    "A1", "A2", "A3", "A4", "A5", "A6", "A7", "A8", "A9",
    "B1", "B2", "B3", "B4", "B5", "B6", "B7", "B8", "B9",
    "C1", "C2", "C3", "C4", "C5", "C6", "C7", "C8", "C9",
    "D1", "D2", "D3", "D4", "D5", "D6", "D7", "D8", "D9",
    "E1", "E2", "E3", "E4", "E5", "E6", "E7", "E8", "E9",
    "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9",
    "G1", "G2", "G3", "G4", "G5", "G6", "G7", "G8", "G9",
    "H1", "H2", "H3", "H4", "H5", "H6", "H7", "H8", "H9",
    "J1", "J2", "J3", "J4", "J5", "J6", "J7", "J8", "J9",
];

/// The coordinates of every cell within its row, column and block.
const COORDS_IN_HOUSES: [[Coord; 3]; 81] = {
    let mut coords = [[Coord::new(0); 3]; 81];
    let mut cell = 0;
    while cell < 81 {
        let row = cell / 9;
        let column = cell % 9;
        let block = 3 * (row % 3) + (column % 3);
        coords[cell as usize] = [Coord::new(column), Coord::new(row), Coord::new(block)];
        cell += 1;
    }
    coords
};

/// Holds the peers for every unique cell.
///
/// A cell's peers are all the cells in the same row, column and block, excluding the cell itself.
const PEERS: [CellSet; 81] = {
    let mut sets: [CellSet; 81] = [CellSet::empty(); 81];
    let mut i = 0;

    while i < 81 {
        let cell = Cell::new(i as u8);
        sets[i] = CellSet::empty()
            .union(cell.row().cells())
            .union(cell.column().cells())
            .union(cell.block().cells())
            .without(cell);
        i += 1;
    }
    sets
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits() {
        assert_eq!(Bit::new(0b1000000), Cell::new(6).bit());
    }
}
