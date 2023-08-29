use std::fmt;
use std::iter::FusedIterator;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Index, Neg, Not, Sub, SubAssign,
};

use crate::layout::CellSet;
use crate::symbols::EMPTY_SET;

use super::{Coord, CoordSet, House, Shape};

const FULL: u16 = (1 << 9) - 1;

#[derive(Clone, Copy, Default, Hash, Eq, PartialEq)]
pub struct HouseSet {
    shape: Shape,
    coords: CoordSet,
}

impl HouseSet {
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

    pub const fn from_bits(shape: Shape, bits: u16) -> Self {
        HouseSet {
            shape,
            coords: CoordSet::from_bits(bits),
        }
    }

    pub const fn from_labels(shape: Shape, labels: &str) -> HouseSet {
        HouseSet {
            shape,
            coords: CoordSet::from_labels(labels),
        }
    }

    pub const fn from_coords(shape: Shape, coords: i32) -> HouseSet {
        HouseSet {
            shape,
            coords: CoordSet::from_coords(coords),
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

    pub const fn size(&self) -> usize {
        self.coords.size()
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
            panic!("Cannot add {} to {} set", house, self.shape);
        }
        self.with_coord(house.coord())
    }

    pub fn with_coord(&self, coord: Coord) -> Self {
        Self {
            shape: self.shape,
            coords: self.coords.with(coord),
        }
    }

    pub fn add(&mut self, house: House) {
        if self.shape != house.shape() {
            panic!("Cannot add {} to {} set", house, self.shape);
        }
        self.add_coord(house.coord());
    }

    pub fn add_coord(&mut self, coord: Coord) {
        self.coords += coord;
    }

    pub fn without(&self, house: House) -> Self {
        if self.shape != house.shape() {
            panic!("Cannot remove {} from {} set", house, self.shape);
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
            panic!("Cannot remove {} from {} set", house, self.shape);
        }
        self.remove_coord(house.coord());
    }

    pub fn remove_coord(&mut self, coord: Coord) {
        self.coords -= coord;
    }

    pub fn union(&self, set: Self) -> Self {
        if self.shape != set.shape() {
            panic!("Cannot compare {} and {} sets", self.shape, set.shape);
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
            panic!("Cannot compare {} and {} sets", self.shape, set.shape);
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
            panic!("Cannot compare {} and {} sets", self.shape, set.shape);
        }
        if self.coords == set.coords {
            Self::empty(self.shape)
        } else {
            Self {
                shape: self.shape,
                coords: self.coords & -set.coords,
            }
        }
    }

    pub fn subtract(&mut self, set: Self) {
        *self = self.minus(set)
    }

    pub fn inverted(&self) -> Self {
        Self {
            shape: self.shape,
            coords: -self.coords,
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
            coords: CoordSet::from_coord(house.coord()),
        }
    }
}

impl From<&str> for HouseSet {
    fn from(labels: &str) -> Self {
        labels.split(' ').map(House::from).union() as HouseSet
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
}

impl<I> HouseIteratorUnion for I
where
    I: Iterator<Item = House>,
{
    fn union(self) -> HouseSet {
        self.fold((true, HouseSet::empty(Shape::Row)), |(first, acc), h| {
            (false, if first { h.into() } else { acc + h })
        })
        .1
    }
}

pub trait HouseSetIteratorUnion {
    fn union(self) -> HouseSet;
}

impl<I> HouseSetIteratorUnion for I
where
    I: Iterator<Item = HouseSet>,
{
    fn union(self) -> HouseSet {
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
    type Output = bool;

    fn not(self) -> bool {
        self.is_empty()
    }
}

impl Neg for HouseSet {
    type Output = Self;

    fn neg(self) -> Self {
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

#[allow(unused_macros)]
macro_rules! houses {
    ($labels:expr) => {{
        HouseSet::from($labels)
    }};
}

#[allow(unused_macros)]
macro_rules! rows {
    ($coords:literal) => {
        HouseSet::from_coords(Shape::Row, $coords)
    };
}

#[allow(unused_macros)]
macro_rules! cols {
    ($labels:literal) => {
        HouseSet::from_coords(Shape::Column, $labels)
    };
}

#[allow(unused_macros)]
macro_rules! blocks {
    ($labels:literal) => {
        HouseSet::from_coords(Shape::Block, $labels)
    };
}

#[allow(unused_imports)]
pub(crate) use {blocks, cols, houses, rows};

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
    use super::*;

    #[test]
    fn empty_has_no_houses() {
        let set = HouseSet::empty(Shape::Row);

        assert!(set.is_empty());
        assert_eq!(0, set.size());
        assert!(set.iter().collect::<Vec<House>>().is_empty());
        House::rows_iter().for_each(|house| assert!(!set.has(house)));
    }

    #[test]
    fn full_has_all_houses() {
        let set = HouseSet::full(Shape::Row);

        assert!(!set.is_empty());
        assert_eq!(9, set.size());
        assert_eq!(9, set.iter().collect::<Vec<House>>().len());
        House::rows_iter().for_each(|house| assert!(set.has(house)));
    }

    #[test]
    fn as_pair_returns_none_if_not_pair() {
        assert!(HouseSet::empty(Shape::Row).as_pair().is_none());
        assert!(HouseSet::full(Shape::Row).as_pair().is_none());
        assert!(HouseSet::from("R2 R4 R8").as_pair().is_none());
    }

    #[test]
    fn as_pair_returns_pair() {
        assert_eq!(
            (House::from("R2"), House::from("R8")),
            HouseSet::from("R2 R8").as_pair().unwrap()
        );
        assert_eq!(
            (House::from("R4"), House::from("R7")),
            HouseSet::from("R7 R4").as_pair().unwrap()
        );
    }

    #[test]
    fn as_triple_returns_none_if_not_triple() {
        assert!(HouseSet::empty(Shape::Row).as_triple().is_none());
        assert!(HouseSet::full(Shape::Row).as_triple().is_none());
        assert!(HouseSet::from("C2 C4 C7 C9").as_triple().is_none());
    }

    #[test]
    fn as_triple_returns_triple() {
        assert_eq!(
            (House::from("C1"), House::from("C2"), House::from("C4")),
            HouseSet::from("C1 C2 C4").as_triple().unwrap()
        );
        assert_eq!(
            (House::from("C1"), House::from("C2"), House::from("C4")),
            HouseSet::from("C4 C1 C2").as_triple().unwrap()
        );
    }
}
