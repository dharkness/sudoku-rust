//! Houses and house sets for unit-based strategy logic.
//!
//! A [`House`] is one row, column, or block. It is identified by a
//! [`Shape`] and a [`Coord`], and provides direct access to its cells and
//! intersections with other houses.
//!
//! [`HouseSet`] and [`CoordSet`] are compact 9-bit sets that make it easy
//! to express patterns like "these two rows" or "all blocks touched by these
//! cells" without allocating.

pub mod coord;
pub mod coord_set;
pub mod house;
pub mod house_set;
pub mod shape;

pub use coord::{Coord, CoordError};
pub use coord_set::CoordSet;
pub use house::{House, HouseIter};
pub use house_set::{HouseIteratorUnion, HouseSet, HouseSetIteratorUnion, Iter};
pub use shape::Shape;
