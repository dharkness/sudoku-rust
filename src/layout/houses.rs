//! [`House`]s are the nine rows, columns and boxes (also called blocks)
//! that make up the [`Board`][crate::puzzle::Board].
//!
//! Each [`House`] has a [`Shape`] and a unique [`Coord`].
//! In a valid puzzle, each `House` must contain exactly one of each [`Known`][crate::layout::Known].

pub mod coord;
pub mod coord_set;
pub mod house;
pub mod house_set;
pub mod shape;

pub use coord::Coord;
pub use coord_set::CoordSet;
pub use house::{House, HouseIter, HousesIter};
pub use house_set::{
    HouseIteratorUnion, HouseSet, HouseSetIteratorIntersection, HouseSetIteratorUnion, Iter,
};
pub use shape::Shape;
