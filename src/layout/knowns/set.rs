use super::Known;
use std::fmt;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Index, Not, Sub, SubAssign,
};

type Size = u16;
type Bits = u16;
type SizeAndBits = u16;

/// A set of knowns implemented using a bit field.
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct KnownSet(SizeAndBits);

const BITS_MASK: Bits = (1 << 9) - 1;
const SIZE_SHIFT: u16 = 16 - 4;
const SIZE_BIT: Bits = 1 << SIZE_SHIFT;

const FULL: SizeAndBits = pack(BITS_MASK, 9);

const MISSING: char = '·';
const EMPTY: &str = "∅";

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

    pub fn from(knowns: &str) -> KnownSet {
        let mut set = KnownSet::empty();
        for c in knowns.chars() {
            set += Known::from(c);
        }
        set
    }

    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub const fn is_full(&self) -> bool {
        self.0 == FULL
    }

    // FACTOR If u128.count_ones() is fast, no need to track size.
    pub const fn size(&self) -> Size {
        (self.0 >> SIZE_SHIFT) as Size
    }

    pub const fn bits(&self) -> Bits {
        self.0 & BITS_MASK
    }

    pub const fn has(&self, known: Known) -> bool {
        self.0 & known.bit() != 0
    }

    pub const fn iter(&self) -> Iter {
        Iter { bits: self.bits() }
    }

    pub fn debug(&self) -> String {
        format!("{:01}:{:09b}", self.size(), self.bits())
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

    fn add(self, rhs: Known) -> KnownSet {
        if self.has(rhs) {
            self
        } else {
            KnownSet(self.0 + rhs.bit() + SIZE_BIT)
        }
    }
}

impl Add<&str> for KnownSet {
    type Output = Self;

    fn add(self, rhs: &str) -> KnownSet {
        self.add(Known::from(rhs))
    }
}

impl AddAssign<Known> for KnownSet {
    fn add_assign(&mut self, rhs: Known) {
        if !self.has(rhs) {
            self.0 += rhs.bit() + SIZE_BIT
        }
    }
}

impl AddAssign<&str> for KnownSet {
    fn add_assign(&mut self, rhs: &str) {
        self.add_assign(Known::from(rhs))
    }
}

impl Sub<Known> for KnownSet {
    type Output = Self;

    fn sub(self, rhs: Known) -> KnownSet {
        if !self.has(rhs) {
            self
        } else {
            KnownSet(self.0 - rhs.bit() - SIZE_BIT)
        }
    }
}

impl SubAssign<Known> for KnownSet {
    fn sub_assign(&mut self, rhs: Known) {
        if self.has(rhs) {
            self.0 -= rhs.bit() + SIZE_BIT
        }
    }
}

impl Not for KnownSet {
    type Output = Self;

    fn not(self) -> KnownSet {
        match self.0 {
            0 => KnownSet(FULL),
            FULL => KnownSet(0),
            _ => KnownSet::new(!self.0 & BITS_MASK),
        }
    }
}

impl BitOr for KnownSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> KnownSet {
        if self == rhs {
            self
        } else {
            KnownSet::new((self.0 | rhs.0) & BITS_MASK)
        }
    }
}

impl BitOrAssign for KnownSet {
    fn bitor_assign(&mut self, rhs: Self) {
        if self.0 != rhs.0 {
            *self = KnownSet::new((self.0 | rhs.0) & BITS_MASK)
        }
    }
}

impl BitAnd for KnownSet {
    type Output = Self;

    fn bitand(self, rhs: Self) -> KnownSet {
        if self == rhs {
            self
        } else {
            KnownSet::new((self.0 & rhs.0) & BITS_MASK)
        }
    }
}

impl BitAndAssign for KnownSet {
    fn bitand_assign(&mut self, rhs: Self) {
        if self.0 != rhs.0 {
            *self = KnownSet::new((self.0 & rhs.0) & BITS_MASK)
        }
    }
}

impl Sub for KnownSet {
    type Output = Self;

    fn sub(self, rhs: Self) -> KnownSet {
        KnownSet::new((self.0 & !rhs.0) & BITS_MASK)
    }
}

impl SubAssign for KnownSet {
    fn sub_assign(&mut self, rhs: Self) {
        *self = KnownSet::new((self.0 & !rhs.0) & BITS_MASK)
    }
}

impl fmt::Display for KnownSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            write!(f, "{}", EMPTY)
        } else {
            let mut s = String::with_capacity(2 + 9);
            s.push('(');
            for k in 0..9 {
                let known = Known::from(k);
                if self.has(known) {
                    s.push(known.label());
                } else {
                    s.push(MISSING)
                }
            }
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

macro_rules! knowns {
    ($s:expr) => {{
        KnownSet::from($s)
    }};
}

pub(crate) use knowns;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_an_empty_set() {
        let set = KnownSet::empty();

        assert!(set.is_empty());
        assert_eq!(0, set.size());
        for i in 0..9 {
            assert!(!set[Known::from(i)]);
        }
    }

    #[test]
    fn full_returns_a_full_set() {
        let set = KnownSet::full();

        assert!(!set.is_empty());
        assert_eq!(9, set.size());
        for i in 0..9 {
            assert!(set[Known::from(i)]);
        }
    }

    #[test]
    fn new_returns_a_set_with_the_given_bits() {
        let set = KnownSet::new(0b101010101);

        assert!(!set.is_empty());
        assert_eq!(5, set.size());
        for i in 0..9 {
            assert_eq!(i % 2 == 0, set[Known::from(i)]);
        }
    }

    #[test]
    fn add_returns_the_same_set_when_the_known_is_present() {
        let set = KnownSet::from("2589");

        let got = set + Known::from("5");
        assert_eq!(set, got);
    }

    #[test]
    fn add_returns_a_new_set_when_the_known_is_not_present() {
        let set = KnownSet::from("2589");

        let got = set + Known::from("6");
        assert_ne!(set, got);
        assert!(got[Known::from("6")]);
    }

    #[test]
    fn sub_returns_the_same_set_when_the_known_is_not_present() {
        let set = KnownSet::from("2589");

        let got = set - Known::from("6");
        assert_eq!(set, got);
    }

    #[test]
    fn sub_returns_the_same_set_when_the_known_is_present() {
        let set = KnownSet::from("2589");

        let got = set - Known::from("5");
        assert_ne!(set, got);
        assert!(!got[Known::from("5")]);
    }

    #[test]
    fn not_returns_an_inverted_set() {
        assert_eq!(KnownSet::full(), !KnownSet::empty());
        assert_eq!(KnownSet::empty(), !KnownSet::full());

        assert_eq!(KnownSet::from("2589"), !KnownSet::from("13467"));
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

        assert_eq!(EMPTY, set.to_string());

        set += "4";
        set += "2";
        set += "6";
        set += "9";

        assert_eq!("(·2·4·6··9)", set.to_string());
    }
}
