//! Provides [`Known`] and [`KnownSet`] to track collections of knowns and methods to manipulate them.

pub mod known;
pub mod known_set;
pub mod value;

pub use known::Known;
pub use known_set::{
    KnownIteratorUnion, KnownSet, KnownSetIteratorIntersection, KnownSetIteratorUnion,
};
pub use value::Value;
