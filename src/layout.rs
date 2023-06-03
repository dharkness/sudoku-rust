//! Combines the various pieces that make up a Sudoku puzzle.

pub mod cells;
pub mod houses;
pub mod values;

pub use cells::{
    Bits, Cell, CellIteratorUnion, CellSet, CellSetIteratorIntersection, CellSetIteratorUnion,
    Rectangle,
};
pub use houses::{
    Coord, House, HouseIteratorUnion, HouseSet, HouseSetIteratorIntersection,
    HouseSetIteratorUnion, Shape,
};
pub use values::{
    Known, KnownIteratorUnion, KnownSet, KnownSetIteratorIntersection, KnownSetIteratorUnion, Value,
};
