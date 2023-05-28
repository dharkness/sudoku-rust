// While the tuple struct is a thin wrapper (should be same memory storage),
// the fact that it's a struct means it cannot be passed by value without moving it.
//
// Or maybe not. References are about ownership--not pointers.

use std::fmt;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Index, Not, Sub, SubAssign,
};

use crate::layout::House;

use super::{Bit, Cell};

pub type Bits = u128;
type Size = u8;
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

    pub const fn of<const N: usize>(cells: &[Cell; N]) -> Set {
        let mut bits: Bits = 0;
        let mut i = 0;

        while i < N {
            bits |= cells[i].bit().bit();
            i += 1;
        }
        Set::new(bits)
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

    pub const fn has(&self, cell: Cell) -> bool {
        self.0 & cell.bit().bit() != 0
    }

    pub const fn with(&self, cell: Cell) -> Set {
        if self.has(cell) {
            return *self;
        }
        let mut copy = *self;
        copy.0 += cell.bit().bit() + SIZE_BIT;
        copy
    }

    pub fn add(&mut self, cell: Cell) {
        if !self.has(cell) {
            self.0 += cell.bit().bit() + SIZE_BIT
        }
    }

    pub const fn without(&self, cell: Cell) -> Set {
        if !self.has(cell) {
            return *self;
        }
        let mut copy = *self;
        copy.0 -= cell.bit().bit() + SIZE_BIT;
        copy
    }

    pub fn remove(&mut self, cell: Cell) {
        if self.has(cell) {
            self.0 -= cell.bit().bit() + SIZE_BIT
        }
    }

    pub const fn union(&self, set: Self) -> Set {
        if self.0 == set.0 {
            *self
        } else {
            Set::new((self.0 | set.0) & CELLS_MASK)
        }
    }

    pub fn union_with(&mut self, set: Self) {
        *self = self.union(set)
    }

    pub const fn intersect(&self, set: Self) -> Set {
        if self.0 == set.0 {
            *self
        } else {
            Set::new((self.0 & set.0) & CELLS_MASK)
        }
    }

    pub fn intersect_with(&mut self, set: Self) {
        *self = self.intersect(set)
    }

    pub const fn minus(&self, set: Self) -> Set {
        if self.0 == set.0 {
            Set::empty()
        } else {
            Set::new((self.0 & !set.0) & CELLS_MASK)
        }
    }

    pub fn subtract(&mut self, set: Self) {
        *self = self.minus(set)
    }

    pub const fn inverted(&self) -> Set {
        match self.0 {
            0 => Set::full(),
            FULL => Set::empty(),
            _ => Set::new(!self.0 & CELLS_MASK),
        }
    }

    pub fn invert(&mut self) {
        *self = self.inverted()
    }

    pub const fn iter(&self) -> Iter {
        Iter {
            iter: self.bit_iter(),
        }
    }

    pub const fn bit_iter(&self) -> BitIter {
        BitIter { bits: self.bits() }
    }

    pub fn debug(&self) -> String {
        format!("{:02}:{:081b}", self.size(), self.bits())
    }
}

impl From<House> for Set {
    fn from(house: House) -> Set {
        house.cells()
    }
}

impl Index<Bit> for Set {
    type Output = bool;

    fn index(&self, bit: Bit) -> &bool {
        if self.has(bit.cell()) {
            &true
        } else {
            &false
        }
    }
}

impl Index<Cell> for Set {
    type Output = bool;

    fn index(&self, cell: Cell) -> &bool {
        if self.has(cell) {
            &true
        } else {
            &false
        }
    }
}

impl Add<Bit> for Set {
    type Output = Self;

    fn add(self, rhs: Bit) -> Set {
        self.with(rhs.cell())
    }
}

impl Add<Cell> for Set {
    type Output = Self;

    fn add(self, rhs: Cell) -> Set {
        self.with(rhs)
    }
}

impl Add<&str> for Set {
    type Output = Self;

    fn add(self, rhs: &str) -> Set {
        self.with(Cell::from(rhs))
    }
}

impl AddAssign<Bit> for Set {
    fn add_assign(&mut self, rhs: Bit) {
        self.add(rhs.cell())
    }
}

impl AddAssign<Cell> for Set {
    fn add_assign(&mut self, rhs: Cell) {
        self.add(rhs)
    }
}

impl AddAssign<&str> for Set {
    fn add_assign(&mut self, rhs: &str) {
        self.add(Cell::from(rhs))
    }
}

impl Sub<Bit> for Set {
    type Output = Self;

    fn sub(self, rhs: Bit) -> Set {
        self.without(rhs.cell())
    }
}

impl Sub<Cell> for Set {
    type Output = Self;

    fn sub(self, rhs: Cell) -> Set {
        self.without(rhs)
    }
}

impl SubAssign<Bit> for Set {
    fn sub_assign(&mut self, rhs: Bit) {
        self.remove(rhs.cell())
    }
}

impl SubAssign<Cell> for Set {
    fn sub_assign(&mut self, rhs: Cell) {
        self.remove(rhs)
    }
}

impl Not for Set {
    type Output = Self;

    fn not(self) -> Set {
        self.inverted()
    }
}

impl BitOr for Set {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Set {
        self.union(rhs)
    }
}

impl BitOrAssign for Set {
    fn bitor_assign(&mut self, rhs: Self) {
        self.union_with(rhs)
    }
}

impl BitAnd for Set {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Set {
        self.intersect(rhs)
    }
}

impl BitAndAssign for Set {
    fn bitand_assign(&mut self, rhs: Self) {
        self.intersect_with(rhs)
    }
}

impl Sub for Set {
    type Output = Self;

    fn sub(self, rhs: Self) -> Set {
        self.minus(rhs)
    }
}

impl SubAssign for Set {
    fn sub_assign(&mut self, rhs: Self) {
        self.subtract(rhs)
    }
}

impl fmt::Display for Set {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            write!(f, "{}", EMPTY)
        } else {
            let mut s = String::with_capacity(3 * self.size() as usize + 2);
            s.push('(');
            for cell in self.iter() {
                s.push(' ');
                s.push_str(cell.label());
            }
            s.push(' ');
            s.push(')');
            write!(f, "{}", s)
        }
    }
}

pub struct Iter {
    iter: BitIter,
}

impl Iterator for Iter {
    type Item = Cell;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(bit) => Some(bit.cell()),
            None => None,
        }
    }
}

pub struct BitIter {
    bits: Bits,
}

impl Iterator for BitIter {
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
