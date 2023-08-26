use std::fmt;
use std::iter::FusedIterator;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Index, Neg, Not, Sub, SubAssign,
};

use crate::symbols::{EMPTY_SET, MISSING};

use super::Known;

type Size = u16;
type Bits = u16;
type SizeAndBits = u16;

/// A set of knowns implemented using a bit field.
#[derive(Clone, Copy, Default, Hash, Eq, PartialEq)]
pub struct KnownSet(SizeAndBits);

const BITS_MASK: Bits = (1 << 9) - 1;
const SIZE_SHIFT: u16 = 16 - 4;
const SIZE_BIT: Bits = 1 << SIZE_SHIFT;

const FULL: SizeAndBits = pack(BITS_MASK, 9);

const fn pack(knowns: Bits, size: Size) -> SizeAndBits {
    debug_assert!(knowns <= BITS_MASK);
    debug_assert!(size <= 9);
    ((size << SIZE_SHIFT) + knowns) as SizeAndBits
}

impl KnownSet {
    pub const fn empty() -> KnownSet {
        KnownSet(0)
    }

    pub const fn full() -> KnownSet {
        KnownSet(FULL)
    }

    pub const fn new(knowns: Bits) -> KnownSet {
        KnownSet(pack(knowns, knowns.count_ones() as Size))
    }

    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub const fn is_full(&self) -> bool {
        self.0 == FULL
    }

    // FACTOR If u16.count_ones() is fast, no need to track size.
    pub const fn size(&self) -> usize {
        (self.0 >> SIZE_SHIFT) as usize
    }

    pub const fn bits(&self) -> Bits {
        self.0 & BITS_MASK
    }

    pub const fn has(&self, known: Known) -> bool {
        self.0 & known.bit() != 0
    }

    pub const fn has_all(&self, subset: KnownSet) -> bool {
        self.intersect(subset).0 == subset.0
    }

    pub const fn is_subset_of(&self, superset: KnownSet) -> bool {
        self.intersect(superset).0 == self.0
    }

    pub const fn as_single(&self) -> Option<Known> {
        if self.size() != 1 {
            None
        } else {
            Some(Known::from_index(self.bits().trailing_zeros()))
        }
    }

    pub const fn as_pair(&self) -> Option<(Known, Known)> {
        if self.size() != 2 {
            None
        } else {
            let mut bits = self.bits();
            let first = Known::from_index(bits.trailing_zeros());
            bits -= first.bit();
            let second = Known::from_index(bits.trailing_zeros());
            Some((first, second))
        }
    }

    pub const fn as_triple(&self) -> Option<(Known, Known, Known)> {
        if self.size() != 3 {
            None
        } else {
            let mut bits = self.bits();
            let first = Known::from_index(bits.trailing_zeros());
            bits -= first.bit();
            let second = Known::from_index(bits.trailing_zeros());
            bits -= second.bit();
            let third = Known::from_index(bits.trailing_zeros());
            Some((first, second, third))
        }
    }

    pub const fn with(&self, known: Known) -> KnownSet {
        if self.has(known) {
            return *self;
        }
        let mut copy = *self;
        copy.0 += known.bit() + SIZE_BIT;
        copy
    }

    pub fn add(&mut self, known: Known) {
        if !self.has(known) {
            self.0 += known.bit() + SIZE_BIT
        }
    }

    pub const fn without(&self, known: Known) -> KnownSet {
        if !self.has(known) {
            return *self;
        }
        let mut copy = *self;
        copy.0 -= known.bit() + SIZE_BIT;
        copy
    }

    pub fn remove(&mut self, known: Known) {
        if self.has(known) {
            self.0 -= known.bit() + SIZE_BIT
        }
    }

    pub const fn union(&self, set: Self) -> KnownSet {
        if self.0 == set.0 {
            *self
        } else {
            KnownSet::new((self.0 | set.0) & BITS_MASK)
        }
    }

    pub fn union_with(&mut self, set: Self) {
        *self = self.union(set)
    }

    pub const fn intersect(&self, set: Self) -> KnownSet {
        if self.0 == set.0 {
            *self
        } else {
            KnownSet::new((self.0 & set.0) & BITS_MASK)
        }
    }

    pub fn intersect_with(&mut self, set: Self) {
        *self = self.intersect(set)
    }

    pub const fn minus(&self, set: Self) -> KnownSet {
        if self.0 == set.0 {
            KnownSet::empty()
        } else {
            KnownSet::new((self.0 & !set.0) & BITS_MASK)
        }
    }

    pub fn subtract(&mut self, set: Self) {
        *self = self.minus(set)
    }

    pub const fn inverted(&self) -> KnownSet {
        match self.0 {
            0 => KnownSet::full(),
            FULL => KnownSet::empty(),
            _ => KnownSet::new(!self.0 & BITS_MASK),
        }
    }

    pub fn invert(&mut self) {
        *self = self.inverted()
    }

    pub const fn iter(&self) -> Iter {
        Iter { bits: self.bits() }
    }

    pub fn debug(&self) -> String {
        format!("{:01}:{:09b}", self.size(), self.bits())
    }
}

impl From<&str> for KnownSet {
    fn from(labels: &str) -> KnownSet {
        labels.split(' ').map(Known::from).union()
    }
}

impl IntoIterator for KnownSet {
    type Item = Known;
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub trait KnownIteratorUnion {
    fn union(self) -> KnownSet;
}

impl<I> KnownIteratorUnion for I
where
    I: Iterator<Item = Known>,
{
    fn union(self) -> KnownSet {
        self.fold(KnownSet::empty(), |acc, h| acc + h)
    }
}

pub trait KnownSetIteratorUnion {
    fn union(self) -> KnownSet;
}

impl<I> KnownSetIteratorUnion for I
where
    I: Iterator<Item = KnownSet>,
{
    fn union(self) -> KnownSet {
        self.fold(KnownSet::empty(), |acc, h| acc | h)
    }
}

pub trait KnownSetIteratorIntersection {
    fn intersection(self) -> KnownSet;
}

impl<I> KnownSetIteratorIntersection for I
where
    I: Iterator<Item = KnownSet>,
{
    fn intersection(self) -> KnownSet {
        self.fold(KnownSet::full(), |acc, h| acc & h)
    }
}

impl FromIterator<Known> for KnownSet {
    fn from_iter<I: IntoIterator<Item = Known>>(iter: I) -> Self {
        let mut set = KnownSet::empty();
        for known in iter {
            set += known;
        }
        set
    }
}

impl FromIterator<KnownSet> for KnownSet {
    fn from_iter<I: IntoIterator<Item = KnownSet>>(iter: I) -> Self {
        let mut union = KnownSet::empty();
        for set in iter {
            union |= set;
        }
        union
    }
}

impl Index<Known> for KnownSet {
    type Output = bool;

    fn index(&self, known: Known) -> &bool {
        if self.has(known) {
            &true
        } else {
            &false
        }
    }
}

impl Index<&str> for KnownSet {
    type Output = bool;

    fn index(&self, known: &str) -> &bool {
        if self.has(Known::from(known)) {
            &true
        } else {
            &false
        }
    }
}

impl Add<Known> for KnownSet {
    type Output = Self;

    fn add(self, rhs: Known) -> KnownSet {
        self.with(rhs)
    }
}

impl Add<&str> for KnownSet {
    type Output = Self;

    fn add(self, rhs: &str) -> KnownSet {
        self.with(Known::from(rhs))
    }
}

impl AddAssign<Known> for KnownSet {
    fn add_assign(&mut self, rhs: Known) {
        self.add(rhs)
    }
}

impl AddAssign<&str> for KnownSet {
    fn add_assign(&mut self, rhs: &str) {
        self.add(Known::from(rhs))
    }
}

impl Sub<Known> for KnownSet {
    type Output = Self;

    fn sub(self, rhs: Known) -> KnownSet {
        self.without(rhs)
    }
}

impl Sub<&str> for KnownSet {
    type Output = Self;

    fn sub(self, rhs: &str) -> KnownSet {
        self.without(Known::from(rhs))
    }
}

impl SubAssign<Known> for KnownSet {
    fn sub_assign(&mut self, rhs: Known) {
        self.remove(rhs)
    }
}

impl SubAssign<&str> for KnownSet {
    fn sub_assign(&mut self, rhs: &str) {
        self.remove(Known::from(rhs))
    }
}

impl Not for KnownSet {
    type Output = bool;

    fn not(self) -> bool {
        self.is_empty()
    }
}

impl Neg for KnownSet {
    type Output = KnownSet;

    fn neg(self) -> KnownSet {
        self.inverted()
    }
}

impl BitOr for KnownSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> KnownSet {
        self.union(rhs)
    }
}

impl BitOrAssign for KnownSet {
    fn bitor_assign(&mut self, rhs: Self) {
        self.union_with(rhs)
    }
}

impl BitAnd for KnownSet {
    type Output = Self;

    fn bitand(self, rhs: Self) -> KnownSet {
        self.intersect(rhs)
    }
}

impl BitAndAssign for KnownSet {
    fn bitand_assign(&mut self, rhs: Self) {
        self.intersect_with(rhs)
    }
}

impl Sub for KnownSet {
    type Output = Self;

    fn sub(self, rhs: Self) -> KnownSet {
        self.minus(rhs)
    }
}

impl SubAssign for KnownSet {
    fn sub_assign(&mut self, rhs: Self) {
        self.subtract(rhs)
    }
}

impl fmt::Display for KnownSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            write!(f, "{}", EMPTY_SET)
        } else {
            let mut s = String::with_capacity(2 + 9);
            s.push('(');
            Known::iter().for_each(|k| {
                if self.has(k) {
                    s.push(k.label());
                } else {
                    s.push(MISSING)
                }
            });
            s.push(')');
            write!(f, "{}", s)
        }
    }
}

impl fmt::Debug for KnownSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[allow(unused_macros)]
macro_rules! knowns {
    ($s:expr) => {{
        KnownSet::from($s)
    }};
}

#[allow(unused_imports)]
pub(crate) use knowns;

pub struct Iter {
    bits: Bits,
}

impl Iterator for Iter {
    type Item = Known;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            None
        } else {
            let bit = 1 << self.bits.trailing_zeros();
            self.bits &= !bit;
            Some(Known::from(bit.trailing_zeros() as u8))
        }
    }
}

impl FusedIterator for Iter {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::values::known::known;

    #[test]
    fn empty_returns_an_empty_set() {
        let set = KnownSet::empty();

        assert!(set.is_empty());
        assert_eq!(0, set.size());
        for i in 1..=9 {
            assert!(!set[known!(i)]);
        }
    }

    #[test]
    fn full_returns_a_full_set() {
        let set = KnownSet::full();

        assert!(!set.is_empty());
        assert_eq!(9, set.size());
        for i in 1..=9 {
            assert!(set[known!(i)]);
        }
    }

    #[test]
    fn new_returns_a_set_with_the_given_bits() {
        let set = KnownSet::new(0b101010101);

        assert!(!set.is_empty());
        assert_eq!(5, set.size());
        for i in 1..=9 {
            assert_eq!(i % 2 == 1, set[known!(i)]);
        }
    }

    #[test]
    fn as_pair_returns_none_if_not_pair() {
        assert!(KnownSet::empty().as_pair().is_none());
        assert!(KnownSet::full().as_pair().is_none());
        assert!(KnownSet::from("2 5 8 9").as_pair().is_none());
    }

    #[test]
    fn as_pair_returns_pair() {
        assert_eq!(
            (Known::new(2), Known::new(5)),
            KnownSet::from("2 5").as_pair().unwrap()
        );
        assert_eq!(
            (Known::new(1), Known::new(9)),
            KnownSet::from("9 1").as_pair().unwrap()
        );
    }

    #[test]
    fn as_triple_returns_none_if_not_triple() {
        assert!(KnownSet::empty().as_triple().is_none());
        assert!(KnownSet::full().as_triple().is_none());
        assert!(KnownSet::from("2 5 8 9").as_triple().is_none());
    }

    #[test]
    fn as_triple_returns_triple() {
        assert_eq!(
            (Known::new(2), Known::new(5), Known::new(8)),
            KnownSet::from("2 5 8").as_triple().unwrap()
        );
        assert_eq!(
            (Known::new(1), Known::new(5), Known::new(9)),
            KnownSet::from("9 5 1").as_triple().unwrap()
        );
    }

    #[test]
    fn add_returns_the_same_set_when_the_known_is_present() {
        let set = knowns!("2 5 8 9");

        let got = set + known!(5);
        assert_eq!(set, got);
    }

    #[test]
    fn add_returns_a_new_set_when_the_known_is_not_present() {
        let set = knowns!("2 5 8 9");

        let got = set + known!(6);
        assert_ne!(set, got);
        assert!(got[known!(6)]);
    }

    #[test]
    fn sub_returns_the_same_set_when_the_known_is_not_present() {
        let set = KnownSet::from("2 5 8 9");

        let got = set - known!(6);
        assert_eq!(set, got);
    }

    #[test]
    fn sub_returns_the_same_set_when_the_known_is_present() {
        let set = KnownSet::from("2 5 8 9");

        let got = set - known!(5);
        assert_ne!(set, got);
        assert!(!got[known!(5)]);
    }

    #[test]
    fn not() {
        assert!(!KnownSet::empty());
        assert!(KnownSet::full());
    }

    #[test]
    fn neg_returns_an_inverted_set() {
        assert_eq!(KnownSet::full(), -KnownSet::empty());
        assert_eq!(KnownSet::empty(), -KnownSet::full());

        assert_eq!(KnownSet::from("2 5 8 9"), -KnownSet::from("1 3 4 6 7"));
    }

    #[test]
    fn unions() {
        assert_eq!(KnownSet::empty(), KnownSet::empty() | KnownSet::empty());
        assert_eq!(KnownSet::full(), KnownSet::full() | KnownSet::empty());
        assert_eq!(KnownSet::full(), KnownSet::empty() | KnownSet::full());
        assert_eq!(KnownSet::full(), KnownSet::full() | KnownSet::full());

        let mut set = KnownSet::empty();
        set |= KnownSet::full();
        assert!(set.is_full());
    }

    #[test]
    fn intersections() {
        assert_eq!(KnownSet::empty(), KnownSet::empty() & KnownSet::empty());
        assert_eq!(KnownSet::empty(), KnownSet::full() & KnownSet::empty());
        assert_eq!(KnownSet::empty(), KnownSet::empty() & KnownSet::full());
        assert_eq!(KnownSet::full(), KnownSet::full() & KnownSet::full());

        let mut set = KnownSet::full();
        set &= KnownSet::empty();
        assert!(set.is_empty());
    }

    #[test]
    fn differences() {
        assert_eq!(KnownSet::empty(), KnownSet::empty() - KnownSet::empty());
        assert_eq!(KnownSet::full(), KnownSet::full() - KnownSet::empty());
        assert_eq!(KnownSet::empty(), KnownSet::empty() - KnownSet::full());
        assert_eq!(KnownSet::empty(), KnownSet::full() - KnownSet::full());

        let mut set = KnownSet::full();
        set -= KnownSet::full();
        assert!(set.is_empty());
    }

    #[test]
    fn strings() {
        let mut set = KnownSet::empty();

        assert_eq!(EMPTY_SET, set.to_string());

        set += "4";
        set += "2";
        set += "6";
        set += "9";

        assert_eq!("(·2·4·6··9)", set.to_string());
    }
}
