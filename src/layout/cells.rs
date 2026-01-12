//! Provides [`Cell`] and [`CellSet`] to track collections of cells and methods to manipulate them.

pub mod bit;
pub mod cell;
pub mod cell_set;
pub mod rectangle;

pub use bit::Bit;
pub use cell::Cell;
pub use cell_set::{CellIteratorUnion, CellSet, CellSetIteratorUnion};
pub use rectangle::Rectangle;
