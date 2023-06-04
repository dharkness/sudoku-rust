//! Combines the various pieces that make up a Sudoku puzzle.

pub mod cells;
pub mod houses;
pub mod values;

pub use cells::{
    Bits, Cell, CellIteratorUnion, CellSet, CellSetIteratorIntersection, CellSetIteratorUnion,
    Rectangle,
};
pub use houses::{
    Coord, House, HouseIter, HouseIteratorUnion, HouseSet, HouseSetIteratorIntersection,
    HouseSetIteratorUnion, HousesIter, Shape,
};
pub use values::{
    Known, KnownIteratorUnion, KnownSet, KnownSetIteratorIntersection, KnownSetIteratorUnion, Value,
};
