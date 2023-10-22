use std::fmt;
use std::ops::{Add, Neg};

use crate::layout::{Coord, House, Shape};

use super::label::{index_from_label, label_from_index};
use super::{Bit, CellSet};

/// Specifies a single cell by its index from left to right and top to bottom.
#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Cell(u8);

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
        HOUSES[self.usize()]
    }

    pub const fn house(&self, shape: Shape) -> House {
        HOUSES[self.usize()][shape.usize()]
    }

    pub const fn row(&self) -> House {
        HOUSES[self.usize()][Shape::Row.usize()]
    }

    pub const fn row_coord(&self) -> Coord {
        HOUSE_COORDS[self.usize()][Shape::Row.usize()]
    }

    pub const fn column(&self) -> House {
        HOUSES[self.usize()][Shape::Column.usize()]
    }

    pub const fn column_coord(&self) -> Coord {
        HOUSE_COORDS[self.usize()][Shape::Column.usize()]
    }

    pub const fn block(&self) -> House {
        HOUSES[self.usize()][Shape::Block.usize()]
    }

    pub const fn block_coord(&self) -> Coord {
        HOUSE_COORDS[self.usize()][Shape::Block.usize()]
    }

    pub const fn coord_in_row(&self) -> Coord {
        COORDS_IN_HOUSES[self.usize()][Shape::Row.usize()]
    }

    pub const fn coord_in_column(&self) -> Coord {
        COORDS_IN_HOUSES[self.usize()][Shape::Column.usize()]
    }

    pub const fn coord_in_block(&self) -> Coord {
        COORDS_IN_HOUSES[self.usize()][Shape::Block.usize()]
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

    pub const fn sees_any(&self, others: CellSet) -> bool {
        PEERS[self.usize()].has_any(others)
    }

    pub const fn label(&self) -> &'static str {
        label_from_index(self.0)
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

impl From<i32> for Cell {
    fn from(index: i32) -> Self {
        debug_assert!(index >= 0 && index < Cell::COUNT as i32);
        Cell::new(index as u8)
    }
}

impl From<usize> for Cell {
    fn from(index: usize) -> Self {
        debug_assert!(index < Cell::COUNT as usize);
        Cell::new(index as u8)
    }
}

impl From<&str> for Cell {
    fn from(label: &str) -> Self {
        Self(index_from_label(label.to_uppercase().as_str()))
    }
}

impl From<String> for Cell {
    fn from(label: String) -> Self {
        Self(index_from_label(label.to_uppercase().as_str()))
    }
}

impl From<Bit> for Cell {
    fn from(bit: Bit) -> Self {
        bit.cell()
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

#[allow(unused_macros)]
macro_rules! cell {
    ($l:expr) => {
        Cell::from($l)
    };
}

#[allow(unused_imports)]
pub(crate) use cell;

/// The coordinates of every cell's row, column and block.
const HOUSE_COORDS: [[Coord; 3]; 81] = {
    let mut coords = [[Coord::new(0); 3]; 81];
    let mut cell = 0;
    while cell < 81 {
        let row = cell / 9;
        let column = cell % 9;
        let block = (row / 3) * 3 + (column / 3);
        coords[cell as usize] = [Coord::new(row), Coord::new(column), Coord::new(block)];
        cell += 1;
    }
    coords
};

/// Every cell's row, column and block.
const HOUSES: [[House; 3]; 81] = {
    let mut houses = [[House::new(Shape::Row, Coord::new(0)); 3]; 81];
    let mut cell = 0;
    while cell < 81 {
        houses[cell as usize] = [
            House::row(HOUSE_COORDS[cell as usize][Shape::Row.usize()]),
            House::column(HOUSE_COORDS[cell as usize][Shape::Column.usize()]),
            House::block(HOUSE_COORDS[cell as usize][Shape::Block.usize()]),
        ];
        cell += 1;
    }
    houses
};

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
