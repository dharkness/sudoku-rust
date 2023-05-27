use super::Known;
use std::fmt;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Index, Not, Sub, SubAssign,
};

type Size = u16;
type Bits = u16;
type SizeAndBits = u16;

/// A set of knowns implemented using a bit field.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Set(SizeAndBits);

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

impl Set {
    pub const fn empty() -> Set {
        Set(0)
    }

    pub const fn full() -> Set {
        Set(FULL)
    }

    pub const fn new(knowns: Bits) -> Set {
        Set(pack(knowns, knowns.count_ones() as Size))
    }

    pub fn from(knowns: &str) -> Set {
        let mut set = Set::empty();
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

    const fn has(&self, known: Known) -> bool {
        self.0 & (known.bit() as u16) != 0
    }

    pub const fn iter(&self) -> Iter {
        Iter { bits: self.bits() }
    }

    pub fn debug(&self) -> String {
        format!("{:01}:{:09b}", self.size(), self.bits())
    }
}

impl Index<Known> for Set {
    type Output = bool;

    fn index(&self, known: Known) -> &bool {
        if self.has(known) {
            &true
        } else {
            &false
        }
    }
}

impl Add<Known> for Set {
    type Output = Self;

    fn add(self, rhs: Known) -> Set {
        if self.has(rhs) {
            self
        } else {
            Set(self.0 + rhs.bit() + SIZE_BIT)
        }
    }
}

impl Add<&str> for Set {
    type Output = Self;

    fn add(self, rhs: &str) -> Set {
        self.add(Known::from(rhs))
    }
}

impl AddAssign<Known> for Set {
    fn add_assign(&mut self, rhs: Known) {
        if !self.has(rhs) {
            self.0 += rhs.bit() + SIZE_BIT
        }
    }
}

impl AddAssign<&str> for Set {
    fn add_assign(&mut self, rhs: &str) {
        self.add_assign(Known::from(rhs))
    }
}

impl Sub<Known> for Set {
    type Output = Self;

    fn sub(self, rhs: Known) -> Set {
        if !self.has(rhs) {
            self
        } else {
            Set(self.0 - rhs.bit() - SIZE_BIT)
        }
    }
}

impl SubAssign<Known> for Set {
    fn sub_assign(&mut self, rhs: Known) {
        if self.has(rhs) {
            self.0 -= rhs.bit() + SIZE_BIT
        }
    }
}

impl Not for Set {
    type Output = Self;

    fn not(self) -> Set {
        match self.0 {
            0 => Set(FULL),
            FULL => Set(0),
            _ => Set::new(!self.0 & BITS_MASK),
        }
    }
}

impl BitOr for Set {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Set {
        if self == rhs {
            self
        } else {
            Set::new((self.0 | rhs.0) & BITS_MASK)
        }
    }
}

impl BitOrAssign for Set {
    fn bitor_assign(&mut self, rhs: Self) {
        if self.0 != rhs.0 {
            *self = Set::new((self.0 | rhs.0) & BITS_MASK)
        }
    }
}

impl BitAnd for Set {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Set {
        if self == rhs {
            self
        } else {
            Set::new((self.0 & rhs.0) & BITS_MASK)
        }
    }
}

impl BitAndAssign for Set {
    fn bitand_assign(&mut self, rhs: Self) {
        if self.0 != rhs.0 {
            *self = Set::new((self.0 & rhs.0) & BITS_MASK)
        }
    }
}

impl Sub for Set {
    type Output = Self;

    fn sub(self, rhs: Self) -> Set {
        Set::new((self.0 & !rhs.0) & BITS_MASK)
    }
}

impl SubAssign for Set {
    fn sub_assign(&mut self, rhs: Self) {
        *self = Set::new((self.0 & !rhs.0) & BITS_MASK)
    }
}

impl fmt::Display for Set {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            write!(f, "{}", EMPTY)
        } else {
            let mut s = String::with_capacity(2 + 9);
            s.push('(');
            for k in 0..9 {
                let known = Known::from(k);
                if self.has(known) {
                    s.push_str(known.label());
                } else {
                    s.push(MISSING)
                }
            }
            s.push(')');
            write!(f, "{}", s)
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_an_empty_set() {
        let set = Set::empty();

        assert!(set.is_empty());
        assert_eq!(0, set.size());
        for i in 0..9 {
            assert!(!set[Known::from(i)]);
        }
    }

    #[test]
    fn full_returns_a_full_set() {
        let set = Set::full();

        assert!(!set.is_empty());
        assert_eq!(9, set.size());
        for i in 0..9 {
            assert!(set[Known::from(i)]);
        }
    }

    #[test]
    fn new_returns_a_set_with_the_given_bits() {
        let set = Set::new(0b101010101);

        assert!(!set.is_empty());
        assert_eq!(5, set.size());
        for i in 0..9 {
            assert_eq!(i % 2 == 0, set[Known::from(i)]);
        }
    }

    #[test]
    fn add_returns_the_same_set_when_the_known_is_present() {
        let set = Set::from("2589");

        let got = set + Known::from("5");
        assert_eq!(set, got);
    }

    #[test]
    fn add_returns_a_new_set_when_the_known_is_not_present() {
        let set = Set::from("2589");

        let got = set + Known::from("6");
        assert_ne!(set, got);
        assert!(got[Known::from("6")]);
    }

    #[test]
    fn sub_returns_the_same_set_when_the_known_is_not_present() {
        let set = Set::from("2589");

        let got = set - Known::from("6");
        assert_eq!(set, got);
    }

    #[test]
    fn sub_returns_the_same_set_when_the_known_is_present() {
        let set = Set::from("2589");

        let got = set - Known::from("5");
        assert_ne!(set, got);
        assert!(!got[Known::from("5")]);
    }

    #[test]
    fn not_returns_an_inverted_set() {
        assert_eq!(Set::full(), !Set::empty());
        assert_eq!(Set::empty(), !Set::full());

        assert_eq!(Set::from("2589"), !Set::from("13467"));
    }

    #[test]
    fn unions() {
        assert_eq!(Set::empty(), Set::empty() | Set::empty());
        assert_eq!(Set::full(), Set::full() | Set::empty());
        assert_eq!(Set::full(), Set::empty() | Set::full());
        assert_eq!(Set::full(), Set::full() | Set::full());

        let mut set = Set::empty();
        set |= Set::full();
        assert!(set.is_full());
    }

    #[test]
    fn intersections() {
        assert_eq!(Set::empty(), Set::empty() & Set::empty());
        assert_eq!(Set::empty(), Set::full() & Set::empty());
        assert_eq!(Set::empty(), Set::empty() & Set::full());
        assert_eq!(Set::full(), Set::full() & Set::full());

        let mut set = Set::full();
        set &= Set::empty();
        assert!(set.is_empty());
    }

    #[test]
    fn differences() {
        assert_eq!(Set::empty(), Set::empty() - Set::empty());
        assert_eq!(Set::full(), Set::full() - Set::empty());
        assert_eq!(Set::empty(), Set::empty() - Set::full());
        assert_eq!(Set::empty(), Set::full() - Set::full());

        let mut set = Set::full();
        set -= Set::full();
        assert!(set.is_empty());
    }

    #[test]
    fn strings() {
        let mut set = Set::empty();

        assert_eq!(EMPTY, set.to_string());

        set += "4";
        set += "2";
        set += "6";
        set += "9";

        assert_eq!("(·2·4·6··9)", set.to_string());
    }
}
