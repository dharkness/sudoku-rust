//! Digit and value helpers for candidate logic.
//!
//! [`Digit`] represents a value 1 through 9 and is the key used by
//! strategies when inspecting or removing candidates.
//!
//! [`DigitSet`] is a 9-bit set with fast set algebra, used for candidate
//! collections, intersections, and eliminations.
//!
//! [`Value`] wraps a solved digit or an unsolved marker and is used by the
//! board to store cell values.

pub mod digit;
pub mod digit_set;
pub mod value;

pub use digit::Digit;
pub use digit_set::{DigitIteratorUnion, DigitSet, DigitSetIteratorUnion};
pub use value::Value;
