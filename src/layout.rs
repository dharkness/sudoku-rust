//! Defines the individual pieces that combine to produce a Sudoku
//! [`Board`][`crate::puzzle::Board`], holding 81 cells in a 9x9 grid.
//!
//! Each [`Cell`] holds a single [`Value`] which will be [`Known`] if given as a clue
//! or later solved to a digit (1 through 9). Until then, it will be considered unknown.
//!
//! The board uses [`CellSet`]s to track the cells that are given, known,
//! have each known value as a candidate, have N candidates remaining,
//! or have been given or solved to each known.
//! This is an 81-bit bitset, with each bit representing a cell on the board.
//! It uses a 128-bit integer to hold the bits for maximum efficiency and provides
//! basic set-manipulation operations required for the board and strategies.
//!
//! The [`Rectangle`] holds four cells and is used for detecting deadly rectangles and
//! avoidable rectangles and by the Unique Rectangle strategy.
//!
//! The board uses [`KnownSet`]s to track the remaining candidates for each unknown cell.
//! This is a 9-bit bitset, with each bit representing a known value.
//! It has nearly the identical interface and features as [`CellSet`].
//!
//! Cells are grouped into [`House`]s containing 9 cells and defined by its [`Shape`],
//! a row, column, or block. Blocks are 3x3 squares often called boxes, but `box`
//! is a reserved word in Rust. There are 9 houses of each shape.
//!
//! Many strategies use [`HouseSet`]s when iterating over rows, columns, or blocks.
//! This uses [`CoordSet`] (see below) along with a [`Shape`] identifying which houses it holds.
//! It has nearly the identical interface and features as the other sets.
//!
//! The [`Coord`] tracks the location of a cell in each of its houses and the location
//! of each house on the board. As there are 9 of each in all cases, the coord ranges
//! from 1 to 9.
//!
//! [`HouseSet`] uses a [`CoordSet`] to track which houses it contains.
//! This is another 9-bit bitset, with each bit representing one of the nine coordinates.
//! It has nearly the identical interface and features as the other sets.

pub use cells::{
    Cell, CellIteratorUnion, CellSet, CellSetIteratorIntersection, CellSetIteratorUnion, Rectangle,
};
pub use houses::{
    Coord, CoordSet, House, HouseIteratorUnion, HouseSet, HouseSetIteratorIntersection,
    HouseSetIteratorUnion, Shape,
};
pub use values::{
    Known, KnownIteratorUnion, KnownSet, KnownSetIteratorIntersection, KnownSetIteratorUnion, Value,
};

pub mod cells;
pub mod houses;
pub mod values;
