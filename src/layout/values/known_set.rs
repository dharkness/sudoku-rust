use std::fmt;
use std::iter::FusedIterator;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Index, Not, Sub, SubAssign,
};

use crate::io::ordinal_suffix;
use crate::layout::values::known::KnownError;
use crate::symbols::{EMPTY_SET, MISSING};

use super::Known;

type Bits = u16;
type Size = u8;

/// A set of knowns implemented using a bit field.
#[derive(Clone, Copy, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct KnownSet(Bits);

/// Errors that can occur when parsing a known set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KnownSetError {
    InvalidKnown {
        position: usize, // 1-based position in the list of knowns
        error: KnownError,
    },
}

impl fmt::Display for KnownSetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KnownSetError::InvalidKnown { position, error } => {
                write!(f, "{} ({}{})", error, position, ordinal_suffix(*position))
            }
        }
    }
}

impl std::error::Error for KnownSetError {}

const ALL_KNOWNS: std::ops::Range<Size> = 0..Known::COUNT;
const ALL_SET: Bits = (1 << Known::COUNT) - 1;

impl KnownSet {
    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn full() -> Self {
        Self(ALL_SET)
    }

    pub const fn new(bits: Bits) -> Self {
        debug_assert!(bits <= ALL_SET);
        Self(bits)
    }

    pub const fn of(known: Known) -> Self {
        Self::new(known.bit())
    }

    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub const fn is_full(&self) -> bool {
        self.0 == ALL_SET
    }

    pub const fn len(&self) -> usize {
        self.0.count_ones() as usize
    }

    pub const fn bits(&self) -> Bits {
        self.0
    }

    pub const fn has(&self, known: Known) -> bool {
        self.0 & known.bit() != 0
    }

    pub const fn has_any(&self, set: KnownSet) -> bool {
        !self.intersect(set).is_empty()
    }

    pub const fn has_all(&self, subset: KnownSet) -> bool {
        self.intersect(subset).0 == subset.0
    }

    pub const fn is_subset_of(&self, superset: KnownSet) -> bool {
        self.intersect(superset).0 == self.0
    }

    pub const fn as_single(&self) -> Option<Known> {
        if self.len() != 1 {
            None
        } else {
            Some(Known::new(self.bits().trailing_zeros()))
        }
    }

    pub const fn as_pair(&self) -> Option<(Known, Known)> {
        if self.len() != 2 {
            None
        } else {
            let mut bits = self.bits();
            let first = Known::new(bits.trailing_zeros());
            bits -= first.bit();
            let second = Known::new(bits.trailing_zeros());
            Some((first, second))
        }
    }

    pub const fn as_triple(&self) -> Option<(Known, Known, Known)> {
        if self.len() != 3 {
            None
        } else {
            let mut bits = self.bits();
            let first = Known::new(bits.trailing_zeros());
            bits -= first.bit();
            let second = Known::new(bits.trailing_zeros());
            bits -= second.bit();
            let third = Known::new(bits.trailing_zeros());
            Some((first, second, third))
        }
    }

    pub const fn with(&self, known: Known) -> Self {
        Self::new(self.0 | known.bit())
    }

    pub fn add(&mut self, known: Known) {
        self.0 |= known.bit();
    }

    pub const fn without(&self, known: Known) -> Self {
        Self::new(self.0 & !(known.bit()))
    }

    pub fn remove(&mut self, known: Known) {
        self.0 &= !(known.bit());
    }

    pub const fn first(&self) -> Option<Known> {
        if self.is_empty() {
            None
        } else {
            Some(Known::new(self.bits().trailing_zeros()))
        }
    }

    pub fn pop(&mut self) -> Option<Known> {
        if self.is_empty() {
            None
        } else {
            let known = Known::new(self.bits().trailing_zeros());
            self.remove(known);
            Some(known)
        }
    }

    pub const fn union(&self, set: Self) -> Self {
        if self.0 == set.0 {
            *self
        } else {
            Self::new(self.0 | set.0)
        }
    }

    pub fn union_with(&mut self, set: Self) {
        *self = self.union(set)
    }

    pub const fn intersect(&self, set: Self) -> Self {
        if self.0 == set.0 {
            *self
        } else {
            Self::new(self.0 & set.0)
        }
    }

    pub fn intersect_with(&mut self, set: Self) {
        *self = self.intersect(set)
    }

    pub const fn minus(&self, set: Self) -> Self {
        if self.0 == set.0 {
            Self::empty()
        } else {
            Self::new(self.0 & !set.0)
        }
    }

    pub fn subtract(&mut self, set: Self) {
        *self = self.minus(set)
    }

    pub const fn inverted(&self) -> Self {
        Self::new(!self.0 & ALL_SET)
    }

    pub fn invert(&mut self) {
        *self = self.inverted()
    }

    pub const fn iter(&self) -> Iter {
        Iter { bits: self.bits() }
    }

    pub fn debug(&self) -> String {
        format!(
            "{:01}:{:09b}",
            self.len(),
            self.bits().reverse_bits() >> (16 - 9)
        )
    }
}

impl From<Known> for KnownSet {
    /// Returns a set containing the single known.
    fn from(known: Known) -> Self {
        KnownSet::empty().with(known)
    }
}

impl From<&Known> for KnownSet {
    /// Returns a set containing the single known.
    fn from(known: &Known) -> Self {
        KnownSet::empty().with(*known)
    }
}

impl TryFrom<&str> for KnownSet {
    type Error = KnownSetError;

    fn try_from(values: &str) -> Result<Self, Self::Error> {
        if values.is_empty() {
            return Ok(Self::empty());
        }

        let cleaned = values.replace([' ', ','], "");

        let knowns: Vec<Known> = cleaned
            .chars()
            .enumerate()
            .map(|(index, ch)| {
                ch.to_string()
                    .parse::<Known>()
                    .map_err(|error| KnownSetError::InvalidKnown {
                        position: index + 1,
                        error,
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(knowns.into_iter().union_knowns())
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
    fn union_knowns(self) -> KnownSet;
}

impl<I> KnownIteratorUnion for I
where
    I: Iterator<Item = Known>,
{
    fn union(self) -> KnownSet {
        self.union_knowns()
    }

    fn union_knowns(self) -> KnownSet {
        self.fold(KnownSet::empty(), |acc, h| acc + h)
    }
}

pub trait KnownSetIteratorUnion {
    fn union(self) -> KnownSet;
    fn union_knowns(self) -> KnownSet;
}

impl<I> KnownSetIteratorUnion for I
where
    I: Iterator<Item = KnownSet>,
{
    fn union(self) -> KnownSet {
        self.union_knowns()
    }

    fn union_knowns(self) -> KnownSet {
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
        let mut set = Self::empty();
        for known in iter {
            set += known;
        }
        set
    }
}

impl FromIterator<KnownSet> for KnownSet {
    fn from_iter<I: IntoIterator<Item = KnownSet>>(iter: I) -> Self {
        let mut union = Self::empty();
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

impl Add<Known> for KnownSet {
    type Output = Self;

    fn add(self, rhs: Known) -> Self {
        self.with(rhs)
    }
}

impl AddAssign<Known> for KnownSet {
    fn add_assign(&mut self, rhs: Known) {
        self.add(rhs)
    }
}

impl Sub<Known> for KnownSet {
    type Output = Self;

    fn sub(self, rhs: Known) -> Self {
        self.without(rhs)
    }
}

impl SubAssign<Known> for KnownSet {
    fn sub_assign(&mut self, rhs: Known) {
        self.remove(rhs)
    }
}

impl Not for KnownSet {
    type Output = Self;

    fn not(self) -> Self {
        self.inverted()
    }
}

impl BitOr for KnownSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
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

    fn bitand(self, rhs: Self) -> Self {
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

    fn sub(self, rhs: Self) -> Self {
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

/// Creates a [`KnownSet`] containing all 9 knowns.
///
/// This is equivalent to calling [`KnownSet::full()`] but provides
/// consistent macro syntax when used alongside [`knowns!`].
///
/// # Examples
///
/// ```
/// # use sudoku_rust::{KnownSet, all_knowns};
/// let all = all_knowns!();
/// assert_eq!(81, all.len());
/// ```
#[macro_export]
macro_rules! all_knowns {
    () => {
        KnownSet::full()
    };
}

/// Creates a [`KnownSet`] from known value digits.
///
/// Compile-time convenience that panics on invalid input.
/// For runtime parsing with error handling, use [`KnownSet::try_from`].
///
/// # Examples
///
/// ```
/// use sudoku_rust::knowns;
///
/// let empty = knowns![];
/// let set = knowns![1 2 3];
/// let set = knowns![5 9];
/// ```
///
/// # Panics
///
/// Panics if any value is invalid. See [`KnownSet::try_from`] for valid formats.
#[macro_export]
macro_rules! knowns {
    () => {
        KnownSet::empty()
    };

    ($($tokens:tt)+) => {
        match KnownSet::try_from(stringify!($($tokens)+)) {
            Ok(set) => set,
            Err(e) => {
                panic!("knowns![{}]: {}", stringify!($($tokens)+), e)
            }
        }
    };
}

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
            Some(Known::new(bit.trailing_zeros()))
        }
    }
}

impl FusedIterator for Iter {}

#[cfg(test)]
mod tests {
    use crate::symbols::EMPTY_SET_STR;
    use crate::*;

    use super::*;

    #[test]
    fn empty_returns_an_empty_set() {
        let set = knowns![];

        assert!(set.is_empty());
        assert_eq!(0, set.len());
        for i in 1..=9 {
            assert!(!set[Known::from_ordinal(i)]);
        }
    }

    #[test]
    fn full_returns_a_full_set() {
        let set = all_knowns![];

        assert!(!set.is_empty());
        assert_eq!(9, set.len());
        for i in 1..=9 {
            assert!(set[Known::from_ordinal(i)]);
        }
    }

    #[test]
    fn new_returns_a_set_with_the_given_bits() {
        let set = KnownSet::new(0b101010101);

        assert!(!set.is_empty());
        assert_eq!(5, set.len());
        for i in 1..=9 {
            assert_eq!(i % 2 == 1, set[Known::from_ordinal(i)]);
        }
    }

    #[test]
    fn as_pair_returns_none_if_not_pair() {
        assert!(knowns![].as_pair().is_none());
        assert!(all_knowns![].as_pair().is_none());
        assert!(knowns![2 5 8 9].as_pair().is_none());
    }

    #[test]
    fn as_pair_returns_pair() {
        assert_eq!((known!(2), known!(5)), knowns![2 5].as_pair().unwrap());
        assert_eq!((known!(1), known!(9)), knowns![9 1].as_pair().unwrap());
    }

    #[test]
    fn as_triple_returns_none_if_not_triple() {
        assert!(knowns![].as_triple().is_none());
        assert!(all_knowns![].as_triple().is_none());
        assert!(knowns![2 5 8 9].as_triple().is_none());
    }

    #[test]
    fn as_triple_returns_triple() {
        assert_eq!(
            (known!(2), known!(5), known!(8)),
            knowns![2 5 8].as_triple().unwrap()
        );
        assert_eq!(
            (known!(1), known!(5), known!(9)),
            knowns![9 5 1].as_triple().unwrap()
        );
    }

    #[test]
    fn add_returns_the_same_set_when_the_known_is_present() {
        let set = knowns![2 5 8 9];

        let got = set + known!(5);
        assert_eq!(set, got);
    }

    #[test]
    fn add_returns_a_new_set_when_the_known_is_not_present() {
        let set = knowns![2 5 8 9];

        let got = set + known!(6);
        assert_ne!(set, got);
        assert!(got[known!(6)]);
    }

    #[test]
    fn sub_returns_the_same_set_when_the_known_is_not_present() {
        let set = knowns![2 5 8 9];

        let got = set - known!(6);
        assert_eq!(set, got);
    }

    #[test]
    fn sub_returns_the_same_set_when_the_known_is_present() {
        let set = knowns![2 5 8 9];

        let got = set - known!(5);
        assert_ne!(set, got);
        assert!(!got[known!(5)]);
    }

    #[test]
    fn not_returns_an_inverted_set() {
        assert_eq!(all_knowns![], !knowns![]);
        assert_eq!(knowns![], !all_knowns![]);

        assert_eq!(knowns![2 5 8 9], !knowns![1 3 4 6 7]);
    }

    #[test]
    fn unions() {
        assert_eq!(knowns![], knowns![] | knowns![]);
        assert_eq!(all_knowns![], all_knowns![] | knowns![]);
        assert_eq!(all_knowns![], knowns![] | all_knowns![]);
        assert_eq!(all_knowns![], all_knowns![] | all_knowns![]);

        let mut set = knowns![];
        set |= all_knowns![];
        assert!(set.is_full());
    }

    #[test]
    fn intersections() {
        assert_eq!(knowns![], knowns![] & knowns![]);
        assert_eq!(knowns![], all_knowns![] & knowns![]);
        assert_eq!(knowns![], knowns![] & all_knowns![]);
        assert_eq!(all_knowns![], all_knowns![] & all_knowns![]);

        let mut set = all_knowns![];
        set &= knowns![];
        assert!(set.is_empty());
    }

    #[test]
    fn differences() {
        assert_eq!(knowns![], knowns![] - knowns![]);
        assert_eq!(all_knowns![], all_knowns![] - knowns![]);
        assert_eq!(knowns![], knowns![] - all_knowns![]);
        assert_eq!(knowns![], all_knowns![] - all_knowns![]);

        let mut set = all_knowns![];
        set -= all_knowns![];
        assert!(set.is_empty());
    }

    #[test]
    fn strings() {
        let mut set = knowns![];

        assert_eq!(EMPTY_SET_STR, set.to_string());

        set += known!(4);
        set += known!(2);
        set += known!(6);
        set += known!(9);

        assert_eq!("(·2·4·6··9)", set.to_string());
    }
}
