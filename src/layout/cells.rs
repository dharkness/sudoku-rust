//! Cell identifiers and bitset utilities for strategy scans.
//!
//! [`Cell`] is a stable, copyable address for a single square, with helpers
//! for row/column/block membership and peer relationships.
//!
//! [`CellSet`] is an 81-bit set with fast set algebra and deterministic
//! iteration, ideal for expressing strategy patterns and eliminations.
//!
//! [`Rectangle`] groups four cells for rectangle-based techniques.

pub mod bit;
pub mod cell;
pub mod cell_set;
pub mod rectangle;

pub use bit::Bit;
pub use cell::Cell;
pub use cell_set::{CellIteratorUnion, CellSet, CellSetIteratorUnion};
pub use rectangle::Rectangle;
