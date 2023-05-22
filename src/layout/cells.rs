///! Provides Cell and Set to track collections of cells and methods to manipulate them.
mod bit;
mod cell;
mod generate;
mod label;
mod set;

pub use bit::Bit;
pub use cell::Cell;
pub use generate::generate_code_for_neighbors;
pub use set::{Bits, Set};
