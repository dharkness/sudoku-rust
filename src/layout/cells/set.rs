// While the tuple struct is a thin wrapper (should be same memory storage),
// the fact that it's a struct means it cannot be passed by value without moving it.
//
// Or maybe not. References are about ownership--not pointers.

use std::fmt;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Index, Not, Sub, SubAssign,
};

use super::{Bit, Cell};

type Size = u32;
type Bits = u128;
type SizeAndBits = u128;

/// A set of cells implemented using a bit field.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Set(SizeAndBits);

const ALL_CELLS: std::ops::Range<Size> = 0..Cell::COUNT;

const CELLS_MASK: Bits = (1 << Cell::COUNT) - 1;
const SIZE_SHIFT: u32 = 128 - 32;
const SIZE_BIT: Bits = 1 << SIZE_SHIFT;

const FULL: SizeAndBits = pack(Bit::ALL, Cell::COUNT);

const EMPTY: &str = "∅";

const fn pack(bits: Bits, size: Size) -> SizeAndBits {
    debug_assert!(bits <= CELLS_MASK);
    debug_assert!(size <= Cell::COUNT);
    (((size as Bits) << SIZE_SHIFT) + bits) as SizeAndBits
}

impl Set {
    pub const fn empty() -> Set {
        Set(0)
    }

    pub const fn full() -> Set {
        Set(FULL)
    }

    pub const fn new(bits: Bits) -> Set {
        Set(pack(bits, bits.count_ones() as Size))
    }

    const fn packed(size_and_bits: SizeAndBits) -> Set {
        Set(size_and_bits)
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
        self.0 & CELLS_MASK
    }

    const fn has(&self, cell: Bit) -> bool {
        self.0 & cell.bit() != 0
    }

    pub const fn iter(&self) -> Iter {
        Iter { bits: self.bits() }
    }

    pub fn debug(&self) -> String {
        format!("{:02}:{:081b}", self.size(), self.bits())
    }
}

impl Index<Bit> for Set {
    type Output = bool;

    fn index(&self, cell: Bit) -> &bool {
        if self.has(cell) {
            &true
        } else {
            &false
        }
    }
}

impl Index<Cell> for Set {
    type Output = bool;

    fn index(&self, cell: Cell) -> &bool {
        if self.has(cell.bit()) {
            &true
        } else {
            &false
        }
    }
}

impl Add<Bit> for Set {
    type Output = Self;

    fn add(self, rhs: Bit) -> Set {
        if self.has(rhs) {
            self
        } else {
            Set(self.0 + *rhs + SIZE_BIT)
        }
    }
}

impl Add<Cell> for Set {
    type Output = Self;

    fn add(self, rhs: Cell) -> Set {
        self.add(rhs.bit())
    }
}

impl Add<&str> for Set {
    type Output = Self;

    fn add(self, rhs: &str) -> Set {
        self.add(Cell::from(rhs))
    }
}

impl AddAssign<Bit> for Set {
    fn add_assign(&mut self, rhs: Bit) {
        if !self.has(rhs) {
            self.0 += *rhs + SIZE_BIT
        }
    }
}

impl AddAssign<Cell> for Set {
    fn add_assign(&mut self, rhs: Cell) {
        self.add_assign(rhs.bit())
    }
}

impl AddAssign<&str> for Set {
    fn add_assign(&mut self, rhs: &str) {
        self.add_assign(Cell::from(rhs).bit())
    }
}

impl Sub<Bit> for Set {
    type Output = Self;

    fn sub(self, rhs: Bit) -> Set {
        if !self.has(rhs) {
            self
        } else {
            Set(self.0 - *rhs - SIZE_BIT)
        }
    }
}

impl Sub<Cell> for Set {
    type Output = Self;

    fn sub(self, rhs: Cell) -> Set {
        self.sub(rhs.bit())
    }
}

impl SubAssign<Bit> for Set {
    fn sub_assign(&mut self, rhs: Bit) {
        if self.has(rhs) {
            self.0 -= *rhs + SIZE_BIT
        }
    }
}

impl SubAssign<Cell> for Set {
    fn sub_assign(&mut self, rhs: Cell) {
        self.sub_assign(rhs.bit())
    }
}

impl Not for Set {
    type Output = Self;

    fn not(self) -> Set {
        match self.0 {
            0 => Set(FULL),
            FULL => Set(0),
            _ => Set::new(!self.0 & CELLS_MASK),
        }
    }
}

impl BitOr for Set {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Set {
        if self == rhs {
            self
        } else {
            Set::new((self.0 | rhs.0) & CELLS_MASK)
        }
    }
}

impl BitOrAssign for Set {
    fn bitor_assign(&mut self, rhs: Self) {
        if self.0 != rhs.0 {
            *self = Set::new((self.0 | rhs.0) & CELLS_MASK)
        }
    }
}

impl BitAnd for Set {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Set {
        if self == rhs {
            self
        } else {
            Set::new((self.0 & rhs.0) & CELLS_MASK)
        }
    }
}

impl BitAndAssign for Set {
    fn bitand_assign(&mut self, rhs: Self) {
        if self.0 != rhs.0 {
            *self = Set::new((self.0 & rhs.0) & CELLS_MASK)
        }
    }
}

impl Sub for Set {
    type Output = Self;

    fn sub(self, rhs: Self) -> Set {
        Set::new((self.0 & !rhs.0) & CELLS_MASK)
    }
}

impl SubAssign for Set {
    fn sub_assign(&mut self, rhs: Self) {
        *self = Set::new((self.0 & !rhs.0) & CELLS_MASK)
    }
}

impl fmt::Display for Set {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            write!(f, "{}", EMPTY)
        } else {
            let mut s = String::with_capacity(3 * self.size() as usize + 2);
            s.push('(');
            for bit in self.iter() {
                s.push(' ');
                s.push_str(bit.cell().label());
            }
            s.push(' ');
            s.push(')');
            write!(f, "{}", s)
        }
    }
}

pub struct Iter {
    bits: Bits,
}

impl Iterator for Iter {
    type Item = Bit;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            None
        } else {
            let bit = 1 << self.bits.trailing_zeros();
            self.bits &= !bit;
            Some(Bit::new(bit))
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
        for i in ALL_CELLS {
            assert!(!set[Cell::new(i)]);
        }
    }

    #[test]
    fn full_returns_a_full_set() {
        let set = Set::full();

        assert!(!set.is_empty());
        assert_eq!(Cell::COUNT, set.size());
        for i in ALL_CELLS {
            assert!(set[Cell::new(i)]);
        }
    }

    #[test]
    fn new_returns_a_set_with_the_given_bits() {
        let set = Set::new(
            0b101010101010101010101010101010101010101010101010101010101010101010101010101010101,
        );

        assert!(!set.is_empty());
        assert_eq!(41, set.size());
        for i in ALL_CELLS {
            assert_eq!(i % 2 == 0, set[Cell::new(i)]);
        }
    }

    #[test]
    fn add_returns_the_same_set_when_the_cell_is_present() {
        let set = Set::new(0b10000001000001 as Bits);

        let got = set + Cell::new(6);
        assert_eq!(set, got);
    }

    #[test]
    fn add_returns_a_new_set_when_the_cell_is_not_present() {
        let set = Set::new(0b10000001000001 as Bits);

        let got = set + Cell::new(42);
        assert_ne!(set, got);
        assert!(got[Cell::new(42)]);
    }

    #[test]
    fn sub_returns_the_same_set_when_the_cell_is_not_present() {
        let set = Set::new(0b10000001000001 as Bits);

        let got = set - Cell::new(42);
        assert_eq!(set, got);
    }

    #[test]
    fn sub_returns_the_same_set_when_the_cell_is_present() {
        let set = Set::new(0b10000001000001 as Bits);

        let got = set - Cell::new(6);
        assert_ne!(set, got);
        assert!(!got[Cell::new(6)]);
    }

    #[test]
    fn not_returns_an_inverted_set() {
        assert_eq!(Set::full(), !Set::empty());
        assert_eq!(Set::empty(), !Set::full());

        assert_eq!(
            Set::new(
                0b100101010101010101010101101001011010100110101010101010101101010101010011001010101
            ),
            !Set::new(
                0b011010101010101010101010010110100101011001010101010101010010101010101100110101010
            )
        );
        assert_eq!(
            Set::new(
                0b011010101010101010101010010110100101011001010101010101010010101010101100110101010
            ),
            !Set::new(
                0b100101010101010101010101101001011010100110101010101010101101010101010011001010101
            )
        );
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

        set += "C4";
        set += "B8";
        set += "F5";
        set += "H2";

        assert_eq!("( B8 C4 F5 H2 )", set.to_string());
    }
}
