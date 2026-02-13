//! Geometry and value primitives for strategy authoring.
//!
//! This module defines the lightweight identifiers used by solvers:
//! [`Cell`], [`House`], [`Shape`], [`Coord`], [`Digit`], and [`Value`]. These types
//! are cheap to copy and provide direct access to structural relationships like
//! peers, houses, and coordinates.
//!
//! Strategy logic should primarily operate on the bitset types:
//! [`CellSet`], [`HouseSet`], [`CoordSet`], and [`DigitSet`]. They provide fast
//! union, intersection, and difference operations with deterministic iteration
//! order, which makes it easy to express scans such as "candidate cells in this
//! house" or "all peers of these cells" without allocating.
//!
//! Common helpers:
//! - [`Cell::peers`] and [`CellSet::peers`] for peer scans.
//! - [`House::cells`] and [`HouseSet::cells`] for unit coverage.
//! - [`Rectangle`] for deadly or avoidable rectangle patterns.
//!
//! The types here are pure and do not carry puzzle state. They are the
//! vocabulary strategies use to talk about the board.
//!
//! See [`crate::puzzle::Board`] for stateful access to candidates and values.

pub use cells::{Cell, CellIteratorUnion, CellSet, CellSetIteratorUnion, Rectangle};
pub use houses::{
    Coord, CoordSet, House, HouseIteratorUnion, HouseSet, HouseSetIteratorUnion, Shape,
};
pub use values::{Digit, DigitIteratorUnion, DigitSet, DigitSetIteratorUnion, Value};

pub mod cells;
pub mod houses;
pub mod values;
