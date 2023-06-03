use std::fmt;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Index, Neg, Not, Sub, SubAssign,
};

use crate::layout::{CellSet, Coord, House, Shape};
use crate::symbols::EMPTY_SET;

const FULL: u16 = (1 << 9) - 1;

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct HouseSet {
    shape: Shape,
    coords: u16,
}

impl HouseSet {
    pub const fn empty(shape: Shape) -> Self {
        Self {
            shape: shape,
            coords: 0,
        }
    }

    pub const fn full(shape: Shape) -> Self {
        Self {
            shape,
            coords: FULL,
        }
    }

    pub const fn shape(&self) -> Shape {
        self.shape
    }

    pub const fn is_empty(&self) -> bool {
        self.coords == 0
    }

    pub const fn is_full(&self) -> bool {
        self.coords == FULL
    }

    pub const fn size(&self) -> u32 {
        self.coords.count_ones()
    }

    pub fn has(&self, house: House) -> bool {
        if self.shape != house.shape() {
            panic!("{} cannot be in {} set", house, self.shape);
        }
        self.coords & house.coord().bit() != 0
    }

    pub fn has_coord(&self, coord: Coord) -> bool {
        self.coords & coord.bit() != 0
    }

    pub fn cells(&self) -> CellSet {
        self.iter().fold(CellSet::empty(), |acc, h| acc | h.cells())
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
            coords: self.coords | coord.bit(),
        }
    }

    pub fn add(&mut self, house: House) {
        if self.shape != house.shape() {
            panic!("Cannot add {} to {} set", house, self.shape);
        }
        self.add_coord(house.coord());
    }

    pub fn add_coord(&mut self, coord: Coord) {
        self.coords |= coord.bit();
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
            coords: self.coords & !coord.bit(),
        }
    }

    pub fn remove(&mut self, house: House) {
        if self.shape != house.shape() {
            panic!("Cannot remove {} from {} set", house, self.shape);
        }
        self.remove_coord(house.coord());
    }

    pub fn remove_coord(&mut self, coord: Coord) {
        self.coords &= !coord.bit();
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
                coords: self.coords & !set.coords,
            }
        }
    }

    pub fn subtract(&mut self, set: Self) {
        *self = self.minus(set)
    }

    pub fn inverted(&self) -> Self {
        match self.coords {
            0 => Self::full(self.shape),
            FULL => Self::empty(self.shape),
            _ => Self {
                shape: self.shape,
                coords: FULL & !self.coords,
            },
        }
    }

    pub fn invert(&mut self) {
        *self = self.inverted()
    }

    pub const fn iter(&self) -> Iter {
        Iter {
            shape: self.shape,
            coords: self.coords,
        }
    }

    pub fn debug(&self) -> String {
        format!("{} {}:{:09b}", self.shape, self.size(), self.coords)
    }
}

impl From<House> for HouseSet {
    fn from(house: House) -> Self {
        HouseSet {
            shape: house.shape(),
            coords: house.coord().bit(),
        }
    }
}

impl From<&str> for HouseSet {
    fn from(labels: &str) -> Self {
        let mut first = true;
        labels
            .split(' ')
            .fold(HouseSet::empty(Shape::Row), |set, label| {
                let house = House::from(label);
                if first {
                    first = false;
                    HouseSet::empty(house.shape()) + house
                } else {
                    set + house
                }
            })
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
            write!(f, "{}", EMPTY_SET)
        } else {
            let mut s = String::with_capacity(3 * self.size() as usize + 2);
            s.push('(');
            for house in self.iter() {
                s.push(' ');
                s.push_str(house.label());
            }
            s.push(' ');
            s.push(')');
            write!(f, "{}", s)
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

#[allow(unused_imports)]
pub(crate) use houses;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_has_no_houses() {
        let set = HouseSet::empty(Shape::Row);

        assert!(set.is_empty());
        assert_eq!(0, set.size());
        assert!(set.iter().collect::<Vec<House>>().is_empty());
        House::all_rows()
            .iter()
            .for_each(|house| assert!(!set.has(*house)));
    }

    #[test]
    fn full_has_all_houses() {
        let set = HouseSet::full(Shape::Row);

        assert!(!set.is_empty());
        assert_eq!(9, set.size());
        assert_eq!(9, set.iter().collect::<Vec<House>>().len());
        House::all_rows()
            .iter()
            .for_each(|house| assert!(set.has(*house)));
    }
}
