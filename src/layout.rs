//! Combines the various pieces that make up a Sudoku puzzle.

pub mod cells;
pub mod houses;
pub mod values;

pub use cells::{Bits, Cell, CellSet, Rectangle};
pub use houses::{Coord, House, Shape};
pub use values::{Known, KnownSet, Value};
