//! Combines the various pieces that make up a Sudoku puzzle.

mod cells;
mod houses;
mod knowns;

pub use cells::{Bits, Cell, Rectangle, Set as CellSet};
pub use houses::{Coord, House, Shape};
pub use knowns::{Known, Set as KnownSet};
