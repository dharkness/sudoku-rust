//! Provides [`Digit`] and [`DigitSet`] to track collections of digits
//! and methods to manipulate them.

pub mod digit;
pub mod digit_set;
pub mod value;

pub use digit::Digit;
pub use digit_set::{DigitIteratorUnion, DigitSet, DigitSetIteratorUnion};
pub use value::Value;
