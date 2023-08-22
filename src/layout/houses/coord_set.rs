use std::fmt;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Index, Neg, Not, Sub, SubAssign,
};

use crate::symbols::{EMPTY_SET, MISSING};

use super::Coord;

/// A set of coordinates in a [`House`] implemented using a bit field.
#[derive(Clone, Copy, Default, Hash, Eq, PartialEq)]
pub struct CoordSet(u16);

const FULL: u16 = (1 << 9) - 1;

impl CoordSet {
    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn full() -> Self {
        Self(FULL)
    }

    pub const fn from_coord(coord: Coord) -> Self {
        Self(coord.bit())
    }

    pub const fn from_bits(bits: u16) -> Self {
        Self(bits & FULL)
    }

    pub const fn from_labels(labels: &str) -> Self {
        let bytes = labels.as_bytes();
        let mut bits: u16 = 0;
        let mut i = 0;

        while i < bytes.len() {
            let c = bytes[i] as char;
            debug_assert!('1' <= c && c <= '9');
            bits += 1 << (c as u8 - b'1');
            i += 1;
        }
        Self(bits)
    }

    pub const fn from_coords(mut coords: i32) -> Self {
        let mut bits: u16 = 0;

        while coords > 0 {
            let c = coords % 10;
            coords /= 10;
            bits += 1 << (c - 1);
        }
        Self(bits)
    }

    pub const fn bits(&self) -> u16 {
        self.0
    }

    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub const fn is_full(&self) -> bool {
        self.0 == FULL
    }

    pub const fn size(&self) -> usize {
        self.0.count_ones() as usize
    }

    pub const fn has(&self, coord: Coord) -> bool {
        self.0 & coord.bit() != 0
    }

    pub const fn has_all(&self, subset: CoordSet) -> bool {
        self.intersect(subset).0 == subset.0
    }

    pub const fn is_subset_of(&self, superset: CoordSet) -> bool {
        self.intersect(superset).0 == self.0
    }

    pub const fn as_single(&self) -> Option<Coord> {
        if self.size() != 1 {
            None
        } else {
            Some(Coord::from_index(self.bits().trailing_zeros()))
        }
    }

    pub const fn as_pair(&self) -> Option<(Coord, Coord)> {
        if self.size() != 2 {
            None
        } else {
            let mut bits = self.bits();
            let first = Coord::from_index(bits.trailing_zeros());
            bits -= first.bit();
            let second = Coord::from_index(bits.trailing_zeros());
            Some((first, second))
        }
    }

    pub const fn as_triple(&self) -> Option<(Coord, Coord, Coord)> {
        if self.size() != 3 {
            None
        } else {
            let mut bits = self.bits();
            let first = Coord::from_index(bits.trailing_zeros());
            bits -= first.bit();
            let second = Coord::from_index(bits.trailing_zeros());
            bits -= second.bit();
            let third = Coord::from_index(bits.trailing_zeros());
            Some((first, second, third))
        }
    }

    pub const fn with(&self, coord: Coord) -> Self {
        if self.has(coord) {
            return *self;
        }
        let mut copy = *self;
        copy.0 += coord.bit();
        copy
    }

    pub fn add(&mut self, coord: Coord) {
        if !self.has(coord) {
            self.0 += coord.bit()
        }
    }

    pub const fn without(&self, coord: Coord) -> Self {
        if !self.has(coord) {
            return *self;
        }
        let mut copy = *self;
        copy.0 -= coord.bit();
        copy
    }

    pub fn remove(&mut self, coord: Coord) {
        if self.has(coord) {
            self.0 -= coord.bit()
        }
    }

    pub const fn union(&self, set: Self) -> Self {
        if self.0 == set.0 {
            *self
        } else {
            Self(self.0 | set.0)
        }
    }

    pub fn union_with(&mut self, set: Self) {
        *self = self.union(set)
    }

    pub const fn intersect(&self, set: Self) -> Self {
        if self.0 == set.0 {
            *self
        } else {
            Self(self.0 & set.0)
        }
    }

    pub fn intersect_with(&mut self, set: Self) {
        *self = self.intersect(set)
    }

    pub const fn minus(&self, set: Self) -> Self {
        if self.0 == set.0 {
            Self::empty()
        } else {
            Self(self.0 & !set.0)
        }
    }

    pub fn subtract(&mut self, set: Self) {
        *self = self.minus(set)
    }

    pub const fn inverted(&self) -> Self {
        match self.0 {
            0 => Self::full(),
            FULL => Self::empty(),
            _ => Self(!self.0 & FULL),
        }
    }

    pub fn invert(&mut self) {
        *self = self.inverted()
    }

    pub const fn iter(&self) -> Iter {
        Iter { bits: self.0 }
    }

    pub fn debug(&self) -> String {
        format!("{:01}:{:09b}", self.size(), self.0)
    }
}

impl From<&str> for CoordSet {
    //noinspection RsTypeCheck
    fn from(labels: &str) -> Self {
        labels.split(' ').map(Coord::from).union()
    }
}

impl From<i32> for CoordSet {
    //noinspection RsTypeCheck
    fn from(coords: i32) -> Self {
        Self::from_coords(coords)
    }
}

impl IntoIterator for CoordSet {
    type Item = Coord;
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub trait CoordIteratorUnion {
    fn union(self) -> CoordSet;
}

impl<I> CoordIteratorUnion for I
where
    I: Iterator<Item = Coord>,
{
    fn union(self) -> CoordSet {
        self.fold(CoordSet::empty(), |acc, h| acc + h)
    }
}

pub trait CoordSetIteratorUnion {
    fn union(self) -> CoordSet;
}

impl<I> CoordSetIteratorUnion for I
where
    I: Iterator<Item = CoordSet>,
{
    fn union(self) -> CoordSet {
        self.fold(CoordSet::empty(), |acc, h| acc | h)
    }
}

pub trait CoordSetIteratorIntersection {
    fn intersection(self) -> CoordSet;
}

impl<I> CoordSetIteratorIntersection for I
where
    I: Iterator<Item = CoordSet>,
{
    fn intersection(self) -> CoordSet {
        self.fold(CoordSet::full(), |acc, h| acc & h)
    }
}

impl FromIterator<Coord> for CoordSet {
    fn from_iter<I: IntoIterator<Item = Coord>>(iter: I) -> Self {
        let mut set = Self::empty();
        for coord in iter {
            set += coord;
        }
        set
    }
}

impl FromIterator<Self> for CoordSet {
    fn from_iter<I: IntoIterator<Item = Self>>(iter: I) -> Self {
        let mut union = Self::empty();
        for set in iter {
            union |= set;
        }
        union
    }
}

impl Index<Coord> for CoordSet {
    type Output = bool;

    fn index(&self, coord: Coord) -> &bool {
        if self.has(coord) {
            &true
        } else {
            &false
        }
    }
}

impl Index<char> for CoordSet {
    type Output = bool;

    fn index(&self, coord: char) -> &bool {
        if self.has(Coord::from(coord)) {
            &true
        } else {
            &false
        }
    }
}

impl Add<Coord> for CoordSet {
    type Output = Self;

    fn add(self, rhs: Coord) -> Self {
        self.with(rhs)
    }
}

impl Add<char> for CoordSet {
    type Output = Self;

    fn add(self, rhs: char) -> Self {
        self.with(Coord::from(rhs))
    }
}

impl AddAssign<Coord> for CoordSet {
    fn add_assign(&mut self, rhs: Coord) {
        self.add(rhs)
    }
}

impl AddAssign<char> for CoordSet {
    fn add_assign(&mut self, rhs: char) {
        self.add(Coord::from(rhs))
    }
}

impl Sub<Coord> for CoordSet {
    type Output = Self;

    fn sub(self, rhs: Coord) -> Self {
        self.without(rhs)
    }
}

impl Sub<char> for CoordSet {
    type Output = Self;

    fn sub(self, rhs: char) -> Self {
        self.without(Coord::from(rhs))
    }
}

impl SubAssign<Coord> for CoordSet {
    fn sub_assign(&mut self, rhs: Coord) {
        self.remove(rhs)
    }
}

impl SubAssign<char> for CoordSet {
    fn sub_assign(&mut self, rhs: char) {
        self.remove(Coord::from(rhs))
    }
}

impl Not for CoordSet {
    type Output = bool;

    fn not(self) -> bool {
        self.is_empty()
    }
}

impl Neg for CoordSet {
    type Output = Self;

    fn neg(self) -> Self {
        self.inverted()
    }
}

impl BitOr for CoordSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

impl BitOrAssign for CoordSet {
    fn bitor_assign(&mut self, rhs: Self) {
        self.union_with(rhs)
    }
}

impl BitAnd for CoordSet {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        self.intersect(rhs)
    }
}

impl BitAndAssign for CoordSet {
    fn bitand_assign(&mut self, rhs: Self) {
        self.intersect_with(rhs)
    }
}

impl Sub for CoordSet {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self.minus(rhs)
    }
}

impl SubAssign for CoordSet {
    fn sub_assign(&mut self, rhs: Self) {
        self.subtract(rhs)
    }
}

impl fmt::Display for CoordSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            write!(f, "{}", EMPTY_SET)
        } else {
            let mut s = String::with_capacity(2 + 9);
            s.push('(');
            (0..9).for_each(|c| {
                if self.has(c.into()) {
                    s.push((b'1' + c) as char);
                } else {
                    s.push(MISSING)
                }
            });
            s.push(')');
            write!(f, "{}", s)
        }
    }
}

impl fmt::Debug for CoordSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[allow(unused_macros)]
macro_rules! coords {
    ($s:expr) => {{
        CoordSet::from($s)
    }};
}

#[allow(unused_imports)]
pub(crate) use coords;

pub struct Iter {
    bits: u16,
}

impl Iterator for Iter {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            None
        } else {
            let bit = 1 << self.bits.trailing_zeros();
            self.bits &= !bit;
            Some(Coord::from(bit.trailing_zeros() as u8))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::houses::coord::coord;

    #[test]
    fn empty_returns_an_empty_set() {
        let set = CoordSet::empty();

        assert!(set.is_empty());
        assert_eq!(0, set.size());
        for i in 1..=9 {
            assert!(!set[coord!(i)]);
        }
    }

    #[test]
    fn full_returns_a_full_set() {
        let set = CoordSet::full();

        assert!(!set.is_empty());
        assert_eq!(9, set.size());
        for i in 1..=9 {
            assert!(set[coord!(i)]);
        }
    }

    #[test]
    fn new_returns_a_set_with_the_given_bits() {
        let set = CoordSet(0b101010101);

        assert!(!set.is_empty());
        assert_eq!(5, set.size());
        for i in 1..=9 {
            assert_eq!(i % 2 == 1, set[coord!(i)]);
        }
    }

    #[test]
    fn as_pair_returns_none_if_not_pair() {
        assert!(CoordSet::empty().as_pair().is_none());
        assert!(CoordSet::full().as_pair().is_none());
        assert!(CoordSet::from("2 5 8 9").as_pair().is_none());
    }

    #[test]
    fn as_pair_returns_pair() {
        assert_eq!(
            (Coord::from_digit(2), Coord::from_digit(5)),
            CoordSet::from("2 5").as_pair().unwrap()
        );
        assert_eq!(
            (Coord::from_digit(1), Coord::from_digit(9)),
            CoordSet::from("9 1").as_pair().unwrap()
        );
    }

    #[test]
    fn as_triple_returns_none_if_not_triple() {
        assert!(CoordSet::empty().as_triple().is_none());
        assert!(CoordSet::full().as_triple().is_none());
        assert!(CoordSet::from("2 5 8 9").as_triple().is_none());
    }

    #[test]
    fn as_triple_returns_triple() {
        assert_eq!(
            (
                Coord::from_digit(2),
                Coord::from_digit(5),
                Coord::from_digit(8)
            ),
            CoordSet::from("2 5 8").as_triple().unwrap()
        );
        assert_eq!(
            (
                Coord::from_digit(1),
                Coord::from_digit(5),
                Coord::from_digit(9)
            ),
            CoordSet::from("9 5 1").as_triple().unwrap()
        );
    }

    #[test]
    fn add_returns_the_same_set_when_the_coord_is_present() {
        let set = coords!("2 5 8 9");

        let got = set + coord!(5);
        assert_eq!(set, got);
    }

    #[test]
    fn add_returns_a_new_set_when_the_coord_is_not_present() {
        let set = coords!("2 5 8 9");

        let got = set + coord!(6);
        assert_ne!(set, got);
        assert!(got[coord!(6)]);
    }

    #[test]
    fn sub_returns_the_same_set_when_the_coord_is_not_present() {
        let set = CoordSet::from("2 5 8 9");

        let got = set - coord!(6);
        assert_eq!(set, got);
    }

    #[test]
    fn sub_returns_the_same_set_when_the_coord_is_present() {
        let set = CoordSet::from("2 5 8 9");

        let got = set - coord!(5);
        assert_ne!(set, got);
        assert!(!got[coord!(5)]);
    }

    #[test]
    fn not() {
        assert!(!CoordSet::empty());
        assert!(CoordSet::full());
    }

    #[test]
    fn neg_returns_an_inverted_set() {
        assert_eq!(CoordSet::full(), -CoordSet::empty());
        assert_eq!(CoordSet::empty(), -CoordSet::full());

        assert_eq!(CoordSet::from("2 5 8 9"), -CoordSet::from("1 3 4 6 7"));
    }

    #[test]
    fn unions() {
        assert_eq!(CoordSet::empty(), CoordSet::empty() | CoordSet::empty());
        assert_eq!(CoordSet::full(), CoordSet::full() | CoordSet::empty());
        assert_eq!(CoordSet::full(), CoordSet::empty() | CoordSet::full());
        assert_eq!(CoordSet::full(), CoordSet::full() | CoordSet::full());

        let mut set = CoordSet::empty();
        set |= CoordSet::full();
        assert!(set.is_full());
    }

    #[test]
    fn intersections() {
        assert_eq!(CoordSet::empty(), CoordSet::empty() & CoordSet::empty());
        assert_eq!(CoordSet::empty(), CoordSet::full() & CoordSet::empty());
        assert_eq!(CoordSet::empty(), CoordSet::empty() & CoordSet::full());
        assert_eq!(CoordSet::full(), CoordSet::full() & CoordSet::full());

        let mut set = CoordSet::full();
        set &= CoordSet::empty();
        assert!(set.is_empty());
    }

    #[test]
    fn differences() {
        assert_eq!(CoordSet::empty(), CoordSet::empty() - CoordSet::empty());
        assert_eq!(CoordSet::full(), CoordSet::full() - CoordSet::empty());
        assert_eq!(CoordSet::empty(), CoordSet::empty() - CoordSet::full());
        assert_eq!(CoordSet::empty(), CoordSet::full() - CoordSet::full());

        let mut set = CoordSet::full();
        set -= CoordSet::full();
        assert!(set.is_empty());
    }

    #[test]
    fn strings() {
        let mut set = CoordSet::empty();

        assert_eq!(EMPTY_SET, set.to_string());

        set += '4';
        set += '2';
        set += '6';
        set += '9';

        assert_eq!("(·2·4·6··9)", set.to_string());
    }
}
