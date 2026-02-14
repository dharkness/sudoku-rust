use std::fmt;
use std::iter::FusedIterator;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Index, Not, Sub, SubAssign,
};

use crate::io::ordinal_suffix;
use crate::layout::houses::coord_set::CoordIteratorUnion;
use crate::layout::houses::house::HouseError;
use crate::layout::CellSet;
use crate::symbols::EMPTY_SET;

use super::{Coord, CoordSet, House, Shape};

const FULL: u16 = (1 << 9) - 1;

#[derive(Clone, Copy, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct HouseSet {
    shape: Shape,
    coords: CoordSet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HouseSetError {
    InvalidHouse { position: usize, error: HouseError },
}

impl fmt::Display for HouseSetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HouseSetError::InvalidHouse { position, error } => {
                write!(f, "{} ({}{})", error, position, ordinal_suffix(*position))
            }
        }
    }
}

impl std::error::Error for HouseSetError {}

impl HouseSet {
    pub const fn new(shape: Shape, coords: CoordSet) -> Self {
        Self { shape, coords }
    }

    pub fn try_from_with_shape(shape: Shape, labels: &str) -> Result<Self, HouseSetError> {
        let cleaned = labels.replace([' ', ','], "");
        if cleaned.is_empty() {
            return Ok(Self::new(shape, CoordSet::empty()));
        }

        let coords: Vec<Coord> = cleaned
            .chars()
            .enumerate()
            .map(|(index, ch)| {
                let position = index + 1;
                ch.to_string()
                    .parse::<Coord>()
                    .map_err(|error| HouseSetError::InvalidHouse {
                        position,
                        error: HouseError::InvalidCoord { shape, error },
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self::new(shape, coords.into_iter().union_coords()))
    }

    pub const fn empty(shape: Shape) -> Self {
        Self {
            shape,
            coords: CoordSet::empty(),
        }
    }

    pub const fn full(shape: Shape) -> Self {
        Self {
            shape,
            coords: CoordSet::full(),
        }
    }

    pub const fn shape(&self) -> Shape {
        self.shape
    }

    pub const fn is_empty(&self) -> bool {
        self.coords.is_empty()
    }

    pub const fn is_full(&self) -> bool {
        self.coords.is_full()
    }

    pub const fn len(&self) -> usize {
        self.coords.len()
    }

    pub fn has(&self, house: House) -> bool {
        if self.shape != house.shape() {
            panic!("{} cannot be in {} set", house, self.shape);
        }
        self.coords.has(house.coord())
    }

    pub fn has_coord(&self, coord: Coord) -> bool {
        self.coords.has(coord)
    }

    pub fn has_any(&self, set: HouseSet) -> bool {
        !self.intersect(set).is_empty()
    }

    pub fn has_all(&self, subset: HouseSet) -> bool {
        self.shape == subset.shape && self.intersect(subset).coords == subset.coords
    }

    pub fn is_subset_of(&self, superset: HouseSet) -> bool {
        self.shape == superset.shape && self.intersect(superset).coords == self.coords
    }

    pub fn cells(&self) -> CellSet {
        self.iter().fold(CellSet::empty(), |acc, h| acc | h.cells())
    }

    pub fn as_single(&self) -> Option<House> {
        self.coords
            .as_single()
            .map(|coord| House::new(self.shape, coord))
    }

    pub fn as_pair(&self) -> Option<(House, House)> {
        self.coords.as_pair().map(|(first, second)| {
            (
                House::new(self.shape, first),
                House::new(self.shape, second),
            )
        })
    }

    pub fn as_triple(&self) -> Option<(House, House, House)> {
        self.coords.as_triple().map(|(first, second, third)| {
            (
                House::new(self.shape, first),
                House::new(self.shape, second),
                House::new(self.shape, third),
            )
        })
    }

    pub fn with(&self, house: House) -> Self {
        if self.shape != house.shape() {
            panic!("cannot add {} to {} set", house, self.shape);
        }
        self.with_coord(house.coord())
    }

    pub const fn with_coord(&self, coord: Coord) -> Self {
        Self {
            shape: self.shape,
            coords: self.coords.with(coord),
        }
    }

    pub fn add(&mut self, house: House) {
        if self.shape != house.shape() {
            panic!("cannot add {} to {} set", house, self.shape);
        }
        self.add_coord(house.coord());
    }

    pub fn add_coord(&mut self, coord: Coord) {
        self.coords += coord;
    }

    pub fn without(&self, house: House) -> Self {
        if self.shape != house.shape() {
            panic!("cannot remove {} from {} set", house, self.shape);
        }
        self.without_coord(house.coord())
    }

    pub fn without_coord(&self, coord: Coord) -> Self {
        Self {
            shape: self.shape,
            coords: self.coords.without(coord),
        }
    }

    pub fn remove(&mut self, house: House) {
        if self.shape != house.shape() {
            panic!("cannot remove {} from {} set", house, self.shape);
        }
        self.remove_coord(house.coord());
    }

    pub fn remove_coord(&mut self, coord: Coord) {
        self.coords -= coord;
    }

    pub const fn first(&self) -> Option<House> {
        match self.coords.first() {
            Some(coord) => Some(House::new(self.shape, coord)),
            None => None,
        }
    }

    pub fn pop(&mut self) -> Option<House> {
        match self.coords.first() {
            Some(coord) => {
                self.remove_coord(coord);
                Some(House::new(self.shape, coord))
            }
            None => None,
        }
    }

    pub fn union(&self, set: Self) -> Self {
        if self.shape != set.shape() {
            panic!("cannot compare {} and {} sets", self.shape, set.shape);
        }
        if self.coords == set.coords {
            *self
        } else {
            Self {
                shape: self.shape,
                coords: self.coords | set.coords,
            }
        }
    }

    pub fn union_with(&mut self, set: Self) {
        *self = self.union(set)
    }

    pub fn intersect(&self, set: Self) -> Self {
        if self.shape != set.shape() {
            panic!("cannot compare {} and {} sets", self.shape, set.shape);
        }
        if self.coords == set.coords {
            *self
        } else {
            Self {
                shape: self.shape,
                coords: self.coords & set.coords,
            }
        }
    }

    pub fn intersect_with(&mut self, set: Self) {
        *self = self.intersect(set)
    }

    pub fn minus(&self, set: Self) -> Self {
        if self.shape != set.shape() {
            panic!("cannot compare {} and {} sets", self.shape, set.shape);
        }
        if self.coords == set.coords {
            Self::empty(self.shape)
        } else {
            Self {
                shape: self.shape,
                coords: self.coords & !set.coords,
            }
        }
    }

    pub fn subtract(&mut self, set: Self) {
        *self = self.minus(set)
    }

    pub fn inverted(&self) -> Self {
        Self {
            shape: self.shape,
            coords: !self.coords,
        }
    }

    pub fn invert(&mut self) {
        *self = self.inverted()
    }

    pub const fn iter(&self) -> Iter {
        Iter {
            shape: self.shape,
            coords: self.coords.bits(),
        }
    }

    pub fn debug(&self) -> String {
        format!("{} {}", self.shape, self.coords.debug())
    }
}

impl From<House> for HouseSet {
    fn from(house: House) -> Self {
        HouseSet {
            shape: house.shape(),
            coords: CoordSet::from(house.coord()),
        }
    }
}

impl From<&House> for HouseSet {
    fn from(house: &House) -> Self {
        HouseSet {
            shape: house.shape(),
            coords: CoordSet::from(house.coord()),
        }
    }
}

impl IntoIterator for HouseSet {
    type Item = House;
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub trait HouseIteratorUnion {
    fn union(self) -> HouseSet;
    fn union_houses(self) -> HouseSet;
}

impl<I> HouseIteratorUnion for I
where
    I: Iterator<Item = House>,
{
    fn union(self) -> HouseSet {
        self.union_houses()
    }

    fn union_houses(self) -> HouseSet {
        self.fold((true, HouseSet::empty(Shape::Row)), |(first, acc), h| {
            (false, if first { h.into() } else { acc + h })
        })
        .1
    }
}

pub trait HouseSetIteratorUnion {
    fn union(self) -> HouseSet;
    fn union_houses(self) -> HouseSet;
}

impl<I> HouseSetIteratorUnion for I
where
    I: Iterator<Item = HouseSet>,
{
    fn union(self) -> HouseSet {
        self.union_houses()
    }

    fn union_houses(self) -> HouseSet {
        self.reduce(|acc, set| acc | set)
            .unwrap_or(HouseSet::empty(Shape::Row))
    }
}

pub trait HouseSetIteratorIntersection {
    fn intersection(self) -> HouseSet;
}

impl<I> HouseSetIteratorIntersection for I
where
    I: Iterator<Item = HouseSet>,
{
    fn intersection(self) -> HouseSet {
        self.reduce(|acc, set| acc & set)
            .unwrap_or(HouseSet::empty(Shape::Row))
    }
}

impl FromIterator<House> for HouseSet {
    fn from_iter<I: IntoIterator<Item = House>>(iter: I) -> Self {
        let mut set = HouseSet::empty(Shape::Row);
        let mut first = true;
        for house in iter {
            if first {
                set = HouseSet::empty(house.shape());
                first = false;
            }
            set += house;
        }
        set
    }
}

impl FromIterator<HouseSet> for HouseSet {
    fn from_iter<I: IntoIterator<Item = HouseSet>>(iter: I) -> Self {
        let mut union = HouseSet::empty(Shape::Row);
        let mut first = true;
        for set in iter {
            if first {
                union = set;
                first = false;
            } else {
                union |= set;
            }
        }
        union
    }
}

impl Index<House> for HouseSet {
    type Output = bool;

    fn index(&self, house: House) -> &bool {
        if self.has(house) {
            &true
        } else {
            &false
        }
    }
}

impl Index<Coord> for HouseSet {
    type Output = bool;

    fn index(&self, coord: Coord) -> &bool {
        if self.has_coord(coord) {
            &true
        } else {
            &false
        }
    }
}

impl Add<House> for HouseSet {
    type Output = Self;

    fn add(self, rhs: House) -> Self {
        self.with(rhs)
    }
}

impl Add<Coord> for HouseSet {
    type Output = Self;

    fn add(self, rhs: Coord) -> Self {
        self.with_coord(rhs)
    }
}

impl AddAssign<House> for HouseSet {
    fn add_assign(&mut self, rhs: House) {
        self.add(rhs)
    }
}

impl AddAssign<Coord> for HouseSet {
    fn add_assign(&mut self, rhs: Coord) {
        self.add_coord(rhs)
    }
}

impl Sub<House> for HouseSet {
    type Output = Self;

    fn sub(self, rhs: House) -> Self {
        self.without(rhs)
    }
}

impl Sub<Coord> for HouseSet {
    type Output = Self;

    fn sub(self, rhs: Coord) -> Self {
        self.without_coord(rhs)
    }
}

impl SubAssign<House> for HouseSet {
    fn sub_assign(&mut self, rhs: House) {
        self.remove(rhs)
    }
}

impl SubAssign<Coord> for HouseSet {
    fn sub_assign(&mut self, rhs: Coord) {
        self.remove_coord(rhs)
    }
}

impl Not for HouseSet {
    type Output = Self;

    fn not(self) -> Self {
        self.inverted()
    }
}

impl BitOr for HouseSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

impl BitOrAssign for HouseSet {
    fn bitor_assign(&mut self, rhs: Self) {
        self.union_with(rhs)
    }
}

impl BitAnd for HouseSet {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        self.intersect(rhs)
    }
}

impl BitAndAssign for HouseSet {
    fn bitand_assign(&mut self, rhs: Self) {
        self.intersect_with(rhs)
    }
}

impl Sub for HouseSet {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self.minus(rhs)
    }
}

impl SubAssign for HouseSet {
    fn sub_assign(&mut self, rhs: Self) {
        self.subtract(rhs)
    }
}

impl fmt::Display for HouseSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            write!(f, "{} {}", self.shape.label(), EMPTY_SET)
        } else {
            write!(f, "{} {}", self.shape.label(), self.coords)
        }
    }
}

impl fmt::Debug for HouseSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.debug())
    }
}

/// Creates a [`HouseSet`] of rows from coordinate labels
/// from top to bottom, starting at A or 1.
///
/// # Examples
///
/// ```
/// use sudoku_rust::{layout::{CoordSet, HouseSet, Shape}, rows};
///
/// let empty = rows![];
/// let set = rows![A C G];
/// let set = rows![1 3 7];
/// ```
///
/// # Panics
///
/// Panics if any coordinate is invalid.
#[macro_export]
macro_rules! rows {
    () => {
        HouseSet::new(Shape::Row, CoordSet::empty())
    };

    ($($tokens:tt)+) => {
        match HouseSet::try_from_with_shape(Shape::Row, stringify!($($tokens)+)) {
            Ok(set) => set,
            Err(e) => panic!("rows![{}]: {}", stringify!($($tokens)+), e),
        }
    };
}

/// Creates a [`HouseSet`] of columns from coordinate labels
/// from left to right, starting at 1.
///
/// # Examples
///
/// ```
/// use sudoku_rust::{cols, layout::{CoordSet, HouseSet, Shape}};
///
/// let empty = cols![];
/// let set = cols![1 3 7];
/// ```
///
/// # Panics
///
/// Panics if any coordinate is invalid.
#[macro_export]
macro_rules! cols {
    () => {
        HouseSet::new(Shape::Column, CoordSet::empty())
    };

    ($($tokens:tt)+) => {
        match HouseSet::try_from_with_shape(Shape::Column, stringify!($($tokens)+)) {
            Ok(set) => set,
            Err(e) => panic!("cols![{}]: {}", stringify!($($tokens)+), e),
        }
    };
}

/// Creates a [`HouseSet`] of blocks from coordinate labels
/// from top to bottom, left to right, and starting at 1.
///
/// # Examples
///
/// ```
/// use sudoku_rust::{blocks, layout::{CoordSet, HouseSet, Shape}};
///
/// let empty = blocks![];
/// let set = blocks![1 5 9];
/// ```
///
/// # Panics
///
/// Panics if any coordinate is invalid.
#[macro_export]
macro_rules! blocks {
    () => {
        HouseSet::new(Shape::Block, CoordSet::empty())
    };

    ($($tokens:tt)+) => {
        match HouseSet::try_from_with_shape(Shape::Block, stringify!($($tokens)+)) {
            Ok(set) => set,
            Err(e) => panic!("blocks![{}]: {}", stringify!($($tokens)+), e),
        }
    };
}

pub struct Iter {
    shape: Shape,
    coords: u16,
}

impl Iterator for Iter {
    type Item = House;

    fn next(&mut self) -> Option<Self::Item> {
        if self.coords == 0 {
            None
        } else {
            let coord = self.coords.trailing_zeros() as u8;
            self.coords &= !(1 << coord);
            Some(House::new(self.shape, coord.into()))
        }
    }
}

impl FusedIterator for Iter {}

#[cfg(test)]
mod tests {
    use crate::*;

    use super::*;

    #[test]
    fn empty_has_no_houses() {
        let set = HouseSet::empty(Shape::Row);

        assert!(set.is_empty());
        assert_eq!(0, set.len());
        assert!(set.iter().collect::<Vec<House>>().is_empty());
        House::rows_iter().for_each(|house| assert!(!set.has(house)));
    }

    #[test]
    fn full_has_all_houses() {
        let set = HouseSet::full(Shape::Row);

        assert!(!set.is_empty());
        assert_eq!(9, set.len());
        assert_eq!(9, set.iter().collect::<Vec<House>>().len());
        House::rows_iter().for_each(|house| assert!(set.has(house)));
    }

    #[test]
    fn as_pair_returns_none_if_not_pair() {
        assert!(HouseSet::empty(Shape::Row).as_pair().is_none());
        assert!(HouseSet::full(Shape::Row).as_pair().is_none());
        assert!(rows![B D H].as_pair().is_none());
    }

    #[test]
    fn as_pair_returns_pair() {
        assert_eq!((row!(B), row!(H)), rows![B H].as_pair().unwrap());
        assert_eq!((row!(D), row!(G)), rows![G D].as_pair().unwrap());
    }

    #[test]
    fn as_triple_returns_none_if_not_triple() {
        assert!(HouseSet::empty(Shape::Row).as_triple().is_none());
        assert!(HouseSet::full(Shape::Row).as_triple().is_none());
        assert!(cols![2 4 7 9].as_triple().is_none());
    }

    #[test]
    fn as_triple_returns_triple() {
        assert_eq!(
            (col!(1), col!(2), col!(4)),
            cols![1 2 4].as_triple().unwrap()
        );
        assert_eq!(
            (col!(1), col!(2), col!(4)),
            cols![4 1 2].as_triple().unwrap()
        );
    }

    #[test]
    fn display_and_debug() {
        let empty = HouseSet::empty(Shape::Row);
        assert_eq!("Row ∅", format!("{}", empty));

        let set = rows![A C];
        let display = format!("{}", set);
        assert!(display.starts_with("Row"));
        assert!(display.contains('1'));
        assert!(display.contains('3'));

        let debug = set.debug();
        assert!(debug.starts_with("row "));
    }

    #[test]
    fn try_from_with_shape_errors() {
        let err = HouseSet::try_from_with_shape(Shape::Row, "Z").unwrap_err();
        assert_eq!("invalid coord 'Z' for row (1st)", err.to_string());
    }

    #[test]
    #[should_panic]
    fn has_panics_for_mismatched_shape() {
        rows![A].has(col!(1));
    }

    #[test]
    fn inverted_returns_complement() {
        assert_eq!(rows![C D E F G H J], rows![A B].inverted());
    }
}
