use std::fmt;

use crate::layout::{Coord, House, Shape};

use super::label::{index_from_label, label_from_index};
use super::{Bit, Set};

/// Specifies a single cell by its index from left to right and top to bottom.
#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Cell(u8);

impl Cell {
    pub const COUNT: u8 = 81;

    pub const fn new(index: u8) -> Self {
        debug_assert!(index < Cell::COUNT);
        Self(index)
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

    pub const fn column(&self) -> House {
        HOUSES[self.usize()][Shape::Column.usize()]
    }

    pub const fn block(&self) -> House {
        HOUSES[self.usize()][Shape::Block.usize()]
    }

    pub const fn coord_in_row(&self) -> Coord {
        HOUSES_COORDS[self.usize()][Shape::Row.usize()]
    }

    pub const fn coord_in_column(&self) -> Coord {
        HOUSES_COORDS[self.usize()][Shape::Column.usize()]
    }

    pub const fn coord_in_block(&self) -> Coord {
        HOUSES_COORDS[self.usize()][Shape::Block.usize()]
    }

    pub const fn neighbors(&self) -> Set {
        NEIGHBORS[self.usize()]
    }

    pub const fn label(&self) -> &'static str {
        label_from_index(self.0)
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

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())?;
        Ok(())
    }
}

const HOUSES: [[House; 3]; 81] = {
    let mut houses = [[House::new(Shape::Row, Coord::new(0)); 3]; 81];
    let mut cell = 0;
    while cell < 81 {
        let row = cell / 9;
        let column = cell % 9;
        let block = (row / 3) * 3 + (column / 3);
        houses[cell as usize] = [
            House::row(Coord::new(row)),
            House::column(Coord::new(column)),
            House::block(Coord::new(block)),
        ];
        cell += 1;
    }
    houses
};

const HOUSES_COORDS: [[Coord; 3]; 81] = {
    let mut coords = [[Coord::new(0); 3]; 81];
    let mut cell = 0;
    while cell < 81 {
        let row = cell / 9;
        let column = cell % 9;
        let block = 3 * (row % 3) + (column % 3);
        coords[cell as usize] = [
            Coord::new(column),
            Coord::new(row),
            Coord::new(block),
        ];
        cell += 1;
    }
    coords
};

/// Holds the neighbors for every unique cell.
/// A cell's neighbors are all the cells in the same row, column and block, excluding the cell itself.
const NEIGHBORS: [Set; 81] = {
    let mut sets: [Set; 81] = [Set::empty(); 81];
    let mut i = 0;

    while i < 81 {
        let cell = Cell::new(i as u8);
        sets[i] = Set::empty()
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
