use std::fmt;
use std::iter::FusedIterator;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Index, Not, Sub, SubAssign,
};

use crate::io::ordinal_suffix;
use crate::layout::values::digit::DigitError;
use crate::symbols::{EMPTY_SET, MISSING};

use super::Digit;

type Bits = u16;
type Size = u8;

/// A set of digits implemented using a bit field.
#[derive(Clone, Copy, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct DigitSet(Bits);

/// Errors that can occur when parsing a digit set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DigitSetError {
    InvalidDigit {
        position: usize, // 1-based position in the list of digits
        error: DigitError,
    },
}

impl fmt::Display for DigitSetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DigitSetError::InvalidDigit { position, error } => {
                write!(f, "{} ({}{})", error, position, ordinal_suffix(*position))
            }
        }
    }
}

impl std::error::Error for DigitSetError {}

const ALL_DIGITS: std::ops::Range<Size> = 0..Digit::COUNT;
const ALL_SET: Bits = (1 << Digit::COUNT) - 1;

impl DigitSet {
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

    pub const fn of(digit: Digit) -> Self {
        Self::new(digit.bit())
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

    pub const fn has(&self, digit: Digit) -> bool {
        self.0 & digit.bit() != 0
    }

    pub const fn has_any(&self, set: DigitSet) -> bool {
        !self.intersect(set).is_empty()
    }

    pub const fn has_all(&self, subset: DigitSet) -> bool {
        self.intersect(subset).0 == subset.0
    }

    pub const fn is_subset_of(&self, superset: DigitSet) -> bool {
        self.intersect(superset).0 == self.0
    }

    pub const fn as_single(&self) -> Option<Digit> {
        if self.len() != 1 {
            None
        } else {
            Some(Digit::new(self.bits().trailing_zeros()))
        }
    }

    pub const fn as_pair(&self) -> Option<(Digit, Digit)> {
        if self.len() != 2 {
            None
        } else {
            let mut bits = self.bits();
            let first = Digit::new(bits.trailing_zeros());
            bits -= first.bit();
            let second = Digit::new(bits.trailing_zeros());
            Some((first, second))
        }
    }

    pub const fn as_triple(&self) -> Option<(Digit, Digit, Digit)> {
        if self.len() != 3 {
            None
        } else {
            let mut bits = self.bits();
            let first = Digit::new(bits.trailing_zeros());
            bits -= first.bit();
            let second = Digit::new(bits.trailing_zeros());
            bits -= second.bit();
            let third = Digit::new(bits.trailing_zeros());
            Some((first, second, third))
        }
    }

    pub const fn with(&self, digit: Digit) -> Self {
        Self::new(self.0 | digit.bit())
    }

    pub fn add(&mut self, digit: Digit) {
        self.0 |= digit.bit();
    }

    pub const fn without(&self, digit: Digit) -> Self {
        Self::new(self.0 & !(digit.bit()))
    }

    pub fn remove(&mut self, digit: Digit) {
        self.0 &= !(digit.bit());
    }

    pub const fn first(&self) -> Option<Digit> {
        if self.is_empty() {
            None
        } else {
            Some(Digit::new(self.bits().trailing_zeros()))
        }
    }

    pub fn pop(&mut self) -> Option<Digit> {
        if self.is_empty() {
            None
        } else {
            let digit = Digit::new(self.bits().trailing_zeros());
            self.remove(digit);
            Some(digit)
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

impl From<Digit> for DigitSet {
    /// Returns a set containing the single digit.
    fn from(digit: Digit) -> Self {
        DigitSet::empty().with(digit)
    }
}

impl From<&Digit> for DigitSet {
    /// Returns a set containing the single digit.
    fn from(digit: &Digit) -> Self {
        DigitSet::empty().with(*digit)
    }
}

impl TryFrom<&str> for DigitSet {
    type Error = DigitSetError;

    fn try_from(values: &str) -> Result<Self, Self::Error> {
        if values.is_empty() {
            return Ok(Self::empty());
        }

        let cleaned = values.replace([' ', ','], "");

        let digits: Vec<Digit> = cleaned
            .chars()
            .enumerate()
            .map(|(index, ch)| {
                ch.to_string()
                    .parse::<Digit>()
                    .map_err(|error| DigitSetError::InvalidDigit {
                        position: index + 1,
                        error,
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(digits.into_iter().union_digits())
    }
}

impl IntoIterator for DigitSet {
    type Item = Digit;
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub trait DigitIteratorUnion {
    fn union(self) -> DigitSet;
    fn union_digits(self) -> DigitSet;
}

impl<I> DigitIteratorUnion for I
where
    I: Iterator<Item = Digit>,
{
    fn union(self) -> DigitSet {
        self.union_digits()
    }

    fn union_digits(self) -> DigitSet {
        self.fold(DigitSet::empty(), |acc, h| acc + h)
    }
}

pub trait DigitSetIteratorUnion {
    fn union(self) -> DigitSet;
    fn union_digits(self) -> DigitSet;
}

impl<I> DigitSetIteratorUnion for I
where
    I: Iterator<Item = DigitSet>,
{
    fn union(self) -> DigitSet {
        self.union_digits()
    }

    fn union_digits(self) -> DigitSet {
        self.fold(DigitSet::empty(), |acc, h| acc | h)
    }
}

pub trait DigitSetIteratorIntersection {
    fn intersection(self) -> DigitSet;
}

impl<I> DigitSetIteratorIntersection for I
where
    I: Iterator<Item = DigitSet>,
{
    fn intersection(self) -> DigitSet {
        self.fold(DigitSet::full(), |acc, h| acc & h)
    }
}

impl FromIterator<Digit> for DigitSet {
    fn from_iter<I: IntoIterator<Item = Digit>>(iter: I) -> Self {
        let mut set = Self::empty();
        for digit in iter {
            set += digit;
        }
        set
    }
}

impl FromIterator<DigitSet> for DigitSet {
    fn from_iter<I: IntoIterator<Item = DigitSet>>(iter: I) -> Self {
        let mut union = Self::empty();
        for set in iter {
            union |= set;
        }
        union
    }
}

impl Index<Digit> for DigitSet {
    type Output = bool;

    fn index(&self, digit: Digit) -> &bool {
        if self.has(digit) {
            &true
        } else {
            &false
        }
    }
}

impl Add<Digit> for DigitSet {
    type Output = Self;

    fn add(self, rhs: Digit) -> Self {
        self.with(rhs)
    }
}

impl AddAssign<Digit> for DigitSet {
    fn add_assign(&mut self, rhs: Digit) {
        self.add(rhs)
    }
}

impl Sub<Digit> for DigitSet {
    type Output = Self;

    fn sub(self, rhs: Digit) -> Self {
        self.without(rhs)
    }
}

impl SubAssign<Digit> for DigitSet {
    fn sub_assign(&mut self, rhs: Digit) {
        self.remove(rhs)
    }
}

impl Not for DigitSet {
    type Output = Self;

    fn not(self) -> Self {
        self.inverted()
    }
}

impl BitOr for DigitSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

impl BitOrAssign for DigitSet {
    fn bitor_assign(&mut self, rhs: Self) {
        self.union_with(rhs)
    }
}

impl BitAnd for DigitSet {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        self.intersect(rhs)
    }
}

impl BitAndAssign for DigitSet {
    fn bitand_assign(&mut self, rhs: Self) {
        self.intersect_with(rhs)
    }
}

impl Sub for DigitSet {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self.minus(rhs)
    }
}

impl SubAssign for DigitSet {
    fn sub_assign(&mut self, rhs: Self) {
        self.subtract(rhs)
    }
}

impl fmt::Display for DigitSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            write!(f, "{}", EMPTY_SET)
        } else {
            let mut s = String::with_capacity(2 + 9);
            s.push('(');
            Digit::iter().for_each(|d| {
                if self.has(d) {
                    s.push(d.label());
                } else {
                    s.push(MISSING)
                }
            });
            s.push(')');
            write!(f, "{}", s)
        }
    }
}

impl fmt::Debug for DigitSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

/// Creates a [`DigitSet`] containing all 9 digits.
///
/// This is equivalent to calling [`DigitSet::full()`] but provides
/// consistent macro syntax when used alongside [`digits!`].
///
/// # Examples
///
/// ```
/// # use sudoku_rust::{DigitSet, all_digits};
/// let all = all_digits!();
/// assert_eq!(81, all.len());
/// ```
#[macro_export]
macro_rules! all_digits {
    () => {
        DigitSet::full()
    };
}

/// Creates a [`DigitSet`] from digit values.
///
/// Compile-time convenience that panics on invalid input.
/// For runtime parsing with error handling, use [`DigitSet::try_from`].
///
/// # Examples
///
/// ```
/// use sudoku_rust::digits;
///
/// let empty = digits![];
/// let set = digits![1 2 3];
/// let set = digits![5 9];
/// ```
///
/// # Panics
///
/// Panics if any value is invalid. See [`DigitSet::try_from`] for valid formats.
#[macro_export]
macro_rules! digits {
    () => {
        DigitSet::empty()
    };

    ($($tokens:tt)+) => {
        match DigitSet::try_from(stringify!($($tokens)+)) {
            Ok(set) => set,
            Err(e) => {
                panic!("digits![{}]: {}", stringify!($($tokens)+), e)
            }
        }
    };
}

pub struct Iter {
    bits: Bits,
}

impl Iterator for Iter {
    type Item = Digit;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            None
        } else {
            let bit = 1 << self.bits.trailing_zeros();
            self.bits &= !bit;
            Some(Digit::new(bit.trailing_zeros()))
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
        let set = digits![];

        assert!(set.is_empty());
        assert_eq!(0, set.len());
        for i in 1..=9 {
            assert!(!set[Digit::from_ordinal(i)]);
        }
    }

    #[test]
    fn full_returns_a_full_set() {
        let set = all_digits![];

        assert!(!set.is_empty());
        assert_eq!(9, set.len());
        for i in 1..=9 {
            assert!(set[Digit::from_ordinal(i)]);
        }
    }

    #[test]
    fn new_returns_a_set_with_the_given_bits() {
        let set = DigitSet::new(0b101010101);

        assert!(!set.is_empty());
        assert_eq!(5, set.len());
        for i in 1..=9 {
            assert_eq!(i % 2 == 1, set[Digit::from_ordinal(i)]);
        }
    }

    #[test]
    fn as_pair_returns_none_if_not_pair() {
        assert!(digits![].as_pair().is_none());
        assert!(all_digits![].as_pair().is_none());
        assert!(digits![2 5 8 9].as_pair().is_none());
    }

    #[test]
    fn as_pair_returns_pair() {
        assert_eq!((digit!(2), digit!(5)), digits![2 5].as_pair().unwrap());
        assert_eq!((digit!(1), digit!(9)), digits![9 1].as_pair().unwrap());
    }

    #[test]
    fn as_triple_returns_none_if_not_triple() {
        assert!(digits![].as_triple().is_none());
        assert!(all_digits![].as_triple().is_none());
        assert!(digits![2 5 8 9].as_triple().is_none());
    }

    #[test]
    fn as_triple_returns_triple() {
        assert_eq!(
            (digit!(2), digit!(5), digit!(8)),
            digits![2 5 8].as_triple().unwrap()
        );
        assert_eq!(
            (digit!(1), digit!(5), digit!(9)),
            digits![9 5 1].as_triple().unwrap()
        );
    }

    #[test]
    fn add_returns_the_same_set_when_the_digit_is_present() {
        let set = digits![2 5 8 9];

        let got = set + digit!(5);
        assert_eq!(set, got);
    }

    #[test]
    fn add_returns_a_new_set_when_the_digit_is_not_present() {
        let set = digits![2 5 8 9];

        let got = set + digit!(6);
        assert_ne!(set, got);
        assert!(got[digit!(6)]);
    }

    #[test]
    fn sub_returns_the_same_set_when_the_digit_is_not_present() {
        let set = digits![2 5 8 9];

        let got = set - digit!(6);
        assert_eq!(set, got);
    }

    #[test]
    fn sub_returns_the_same_set_when_the_digit_is_present() {
        let set = digits![2 5 8 9];

        let got = set - digit!(5);
        assert_ne!(set, got);
        assert!(!got[digit!(5)]);
    }

    #[test]
    fn not_returns_an_inverted_set() {
        assert_eq!(all_digits![], !digits![]);
        assert_eq!(digits![], !all_digits![]);

        assert_eq!(digits![2 5 8 9], !digits![1 3 4 6 7]);
    }

    #[test]
    fn unions() {
        assert_eq!(digits![], digits![] | digits![]);
        assert_eq!(all_digits![], all_digits![] | digits![]);
        assert_eq!(all_digits![], digits![] | all_digits![]);
        assert_eq!(all_digits![], all_digits![] | all_digits![]);

        let mut set = digits![];
        set |= all_digits![];
        assert!(set.is_full());
    }

    #[test]
    fn intersections() {
        assert_eq!(digits![], digits![] & digits![]);
        assert_eq!(digits![], all_digits![] & digits![]);
        assert_eq!(digits![], digits![] & all_digits![]);
        assert_eq!(all_digits![], all_digits![] & all_digits![]);

        let mut set = all_digits![];
        set &= digits![];
        assert!(set.is_empty());
    }

    #[test]
    fn differences() {
        assert_eq!(digits![], digits![] - digits![]);
        assert_eq!(all_digits![], all_digits![] - digits![]);
        assert_eq!(digits![], digits![] - all_digits![]);
        assert_eq!(digits![], all_digits![] - all_digits![]);

        let mut set = all_digits![];
        set -= all_digits![];
        assert!(set.is_empty());
    }

    #[test]
    fn strings() {
        let mut set = digits![];

        assert_eq!(EMPTY_SET_STR, set.to_string());

        set += digit!(4);
        set += digit!(2);
        set += digit!(6);
        set += digit!(9);

        assert_eq!("(·2·4·6··9)", set.to_string());
    }
}
