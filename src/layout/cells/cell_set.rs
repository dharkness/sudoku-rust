// While the tuple struct is a thin wrapper (should be same memory storage),
// the fact that it's a struct means it cannot be passed by value without moving it.
//
// Or maybe not. References are about ownership--not pointers.

use std::fmt;
use std::iter::FusedIterator;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Index, Neg, Not, Sub, SubAssign,
};

use crate::layout::{House, HouseSet, Shape};
use crate::symbols::EMPTY_SET;

use super::{Bit, Cell};

type Bits = u128;
type Size = u8;

/// A set of cells implemented using a bit field.
#[derive(Clone, Copy, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct CellSet(Bits);

const ALL_CELLS: std::ops::Range<Size> = 0..Cell::COUNT;
const ALL_SET: Bits = (1 << Cell::COUNT) - 1;

impl CellSet {
    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn full() -> Self {
        Self(ALL_SET)
    }

    const fn new(bits: Bits) -> Self {
        debug_assert!(bits <= ALL_SET);
        Self(bits)
    }

    pub fn new_from_pattern(puzzle: &str) -> Self {
        let mut bits: Bits = 0;
        let mut c = 0;

        for char in puzzle.chars() {
            match char {
                ' ' | '\r' | '\n' | '|' | '_' => continue,
                '1'..='9' => bits |= Cell::new(c).bit().bit(),
                _ => (),
            }
            c += 1;
        }
        CellSet::new(bits)
    }

    pub const fn of<const N: usize>(cells: &[Cell; N]) -> Self {
        let mut bits: Bits = 0;
        let mut i = 0;

        while i < N {
            bits |= cells[i].bit().bit();
            i += 1;
        }
        CellSet::new(bits)
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

    pub const fn has(&self, cell: Cell) -> bool {
        self.0 & cell.bit().bit() != 0
    }

    pub const fn has_any(&self, set: CellSet) -> bool {
        !self.intersect(set).is_empty()
    }

    pub const fn has_all(&self, subset: CellSet) -> bool {
        self.intersect(subset).0 == subset.0
    }

    pub const fn is_subset_of(&self, superset: CellSet) -> bool {
        self.intersect(superset).0 == self.0
    }

    pub const fn as_single(&self) -> Option<Cell> {
        if self.len() != 1 {
            None
        } else {
            Some(Cell::new(self.bits().trailing_zeros() as u8))
        }
    }

    pub const fn as_pair(&self) -> Option<(Cell, Cell)> {
        if self.len() != 2 {
            None
        } else {
            let mut bits = self.bits();
            let first = Cell::new(bits.trailing_zeros() as u8);
            bits -= first.bit().bit();
            let second = Cell::new(bits.trailing_zeros() as u8);
            Some((first, second))
        }
    }

    pub const fn as_triple(&self) -> Option<(Cell, Cell, Cell)> {
        if self.len() != 3 {
            None
        } else {
            let mut bits = self.bits();
            let first = Cell::new(bits.trailing_zeros() as u8);
            bits -= first.bit().bit();
            let second = Cell::new(bits.trailing_zeros() as u8);
            bits -= second.bit().bit();
            let third = Cell::new(bits.trailing_zeros() as u8);
            Some((first, second, third))
        }
    }

    pub const fn with(&self, cell: Cell) -> Self {
        Self::new(self.0 | cell.bit().bit())
    }

    pub fn add(&mut self, cell: Cell) {
        self.0 |= cell.bit().bit();
    }

    pub const fn without(&self, cell: Cell) -> Self {
        Self::new(self.0 & !(cell.bit().bit()))
    }

    pub fn remove(&mut self, cell: Cell) {
        self.0 &= !(cell.bit().bit());
    }

    pub const fn first(&self) -> Option<Cell> {
        if self.is_empty() {
            None
        } else {
            Some(Cell::new(self.bits().trailing_zeros() as u8))
        }
    }

    pub fn pop(&mut self) -> Option<Cell> {
        if self.is_empty() {
            None
        } else {
            let cell = Cell::new(self.bits().trailing_zeros() as u8);
            self.remove(cell);
            Some(cell)
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
        match self.0 {
            0 => CellSet::full(),
            ALL_SET => Self::empty(),
            _ => Self::new(!self.0 & ALL_SET),
        }
    }

    pub fn invert(&mut self) {
        *self = self.inverted()
    }

    pub fn rows(&self) -> HouseSet {
        self.houses(Shape::Row)
    }

    pub fn columns(&self) -> HouseSet {
        self.houses(Shape::Column)
    }

    pub fn blocks(&self) -> HouseSet {
        self.houses(Shape::Block)
    }

    pub fn houses(&self, shape: Shape) -> HouseSet {
        self.iter()
            .fold(HouseSet::empty(shape), |set, cell| set + cell.house(shape))
    }

    pub const fn iter(&self) -> CellIter {
        CellIter {
            iter: self.bit_iter(),
        }
    }

    pub const fn bit_iter(&self) -> BitIter {
        BitIter { bits: self.bits() }
    }

    pub fn pattern_string(&self) -> String {
        (0..Cell::COUNT)
            .map(|i| if self.has(Cell::new(i)) { '1' } else { '.' })
            .collect()
    }

    pub fn debug(&self) -> String {
        format!(
            "{:02}:{:081b}",
            self.len(),
            self.bits().reverse_bits() >> (128 - 81)
        )
    }
}

impl From<House> for CellSet {
    fn from(house: House) -> Self {
        house.cells()
    }
}

impl From<&str> for CellSet {
    fn from(labels: &str) -> Self {
        if labels.is_empty() {
            Self::empty()
        } else {
            labels.split(' ').map(Cell::from).union()
        }
    }
}

impl IntoIterator for CellSet {
    type Item = Cell;
    type IntoIter = CellIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub trait CellIteratorUnion {
    fn union(self) -> CellSet;
}

impl<I> CellIteratorUnion for I
where
    I: Iterator<Item = Cell>,
{
    fn union(self) -> CellSet {
        self.fold(CellSet::empty(), |acc, c| acc + c)
    }
}

pub trait CellSetIteratorUnion {
    fn union(self) -> CellSet;
}

impl<I> CellSetIteratorUnion for I
where
    I: Iterator<Item = CellSet>,
{
    fn union(self) -> CellSet {
        self.fold(CellSet::empty(), |acc, c| acc | c)
    }
}

pub trait CellSetIteratorIntersection {
    fn intersection(self) -> CellSet;
}

impl<I> CellSetIteratorIntersection for I
where
    I: Iterator<Item = CellSet>,
{
    fn intersection(self) -> CellSet {
        self.fold(CellSet::full(), |acc, c| acc & c)
    }
}

impl FromIterator<Cell> for CellSet {
    fn from_iter<I: IntoIterator<Item = Cell>>(iter: I) -> Self {
        let mut set = Self::empty();
        for cell in iter {
            set += cell;
        }
        set
    }
}

impl FromIterator<CellSet> for CellSet {
    fn from_iter<I: IntoIterator<Item = CellSet>>(iter: I) -> Self {
        let mut union = Self::empty();
        for set in iter {
            union |= set;
        }
        union
    }
}

impl Index<Bit> for CellSet {
    type Output = bool;

    fn index(&self, bit: Bit) -> &bool {
        if self.has(bit.cell()) {
            &true
        } else {
            &false
        }
    }
}

impl Index<Cell> for CellSet {
    type Output = bool;

    fn index(&self, cell: Cell) -> &bool {
        if self.has(cell) {
            &true
        } else {
            &false
        }
    }
}

impl Index<&str> for CellSet {
    type Output = bool;

    fn index(&self, cell: &str) -> &bool {
        if self.has(Cell::from(cell)) {
            &true
        } else {
            &false
        }
    }
}

impl Add<Bit> for CellSet {
    type Output = Self;

    fn add(self, rhs: Bit) -> Self {
        self.with(rhs.cell())
    }
}

impl Add<Cell> for CellSet {
    type Output = Self;

    fn add(self, rhs: Cell) -> Self {
        self.with(rhs)
    }
}

impl Add<&str> for CellSet {
    type Output = Self;

    fn add(self, rhs: &str) -> Self {
        self.with(Cell::from(rhs))
    }
}

impl AddAssign<Bit> for CellSet {
    fn add_assign(&mut self, rhs: Bit) {
        self.add(rhs.cell())
    }
}

impl AddAssign<Cell> for CellSet {
    fn add_assign(&mut self, rhs: Cell) {
        self.add(rhs)
    }
}

impl AddAssign<&str> for CellSet {
    fn add_assign(&mut self, rhs: &str) {
        self.add(Cell::from(rhs))
    }
}

impl Sub<Bit> for CellSet {
    type Output = Self;

    fn sub(self, rhs: Bit) -> Self {
        self.without(rhs.cell())
    }
}

impl Sub<Cell> for CellSet {
    type Output = Self;

    fn sub(self, rhs: Cell) -> Self {
        self.without(rhs)
    }
}

impl Sub<&str> for CellSet {
    type Output = Self;

    fn sub(self, rhs: &str) -> Self {
        self.without(Cell::from(rhs))
    }
}

impl SubAssign<Bit> for CellSet {
    fn sub_assign(&mut self, rhs: Bit) {
        self.remove(rhs.cell())
    }
}

impl SubAssign<Cell> for CellSet {
    fn sub_assign(&mut self, rhs: Cell) {
        self.remove(rhs)
    }
}

impl SubAssign<&str> for CellSet {
    fn sub_assign(&mut self, rhs: &str) {
        self.remove(Cell::from(rhs))
    }
}

impl Not for CellSet {
    type Output = bool;

    fn not(self) -> bool {
        self.is_empty()
    }
}

impl Neg for CellSet {
    type Output = Self;

    fn neg(self) -> Self {
        self.inverted()
    }
}

impl BitOr for CellSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

impl BitOrAssign for CellSet {
    fn bitor_assign(&mut self, rhs: Self) {
        self.union_with(rhs)
    }
}

impl BitAnd for CellSet {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        self.intersect(rhs)
    }
}

impl BitAndAssign for CellSet {
    fn bitand_assign(&mut self, rhs: Self) {
        self.intersect_with(rhs)
    }
}

impl Sub for CellSet {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self.minus(rhs)
    }
}

impl SubAssign for CellSet {
    fn sub_assign(&mut self, rhs: Self) {
        self.subtract(rhs)
    }
}

impl fmt::Display for CellSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            write!(f, "{}", EMPTY_SET)
        } else {
            let mut s = String::with_capacity(3 * self.len() + 2);
            let mut first = true;
            for cell in self.iter() {
                if first {
                    first = false;
                } else {
                    s.push(' ');
                }
                s.push_str(cell.label());
            }
            write!(f, "{}", s)
        }
    }
}

impl fmt::Debug for CellSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[allow(unused_macros)]
macro_rules! cells {
    ($s:expr) => {{
        CellSet::from($s)
    }};
}

#[allow(unused_imports)]
pub(crate) use cells;

pub struct CellIter {
    iter: BitIter,
}

impl Iterator for CellIter {
    type Item = Cell;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|bit| bit.cell())
    }
}

impl FusedIterator for CellIter {}

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

impl FusedIterator for BitIter {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_an_empty_set() {
        let set = CellSet::empty();

        assert!(set.is_empty());
        assert_eq!(0, set.len());
        for i in ALL_CELLS {
            assert!(!set[Cell::new(i)]);
        }
    }

    #[test]
    fn full_returns_a_full_set() {
        let set = CellSet::full();

        assert!(!set.is_empty());
        assert_eq!(Cell::COUNT, set.len() as u8);
        for i in ALL_CELLS {
            assert!(set[Cell::new(i)]);
        }
    }

    #[test]
    fn new_returns_a_set_with_the_given_bits() {
        let set = CellSet::new(
            0b101010101010101010101010101010101010101010101010101010101010101010101010101010101,
        );

        assert!(!set.is_empty());
        assert_eq!(41, set.len());
        for i in ALL_CELLS {
            assert_eq!(i % 2 == 0, set[Cell::new(i)]);
        }
    }

    #[test]
    fn as_pair_returns_none_if_not_pair() {
        assert!(CellSet::empty().as_pair().is_none());
        assert!(CellSet::full().as_pair().is_none());
        assert!(CellSet::from("A5 D9 F3 H5").as_pair().is_none());
    }

    #[test]
    fn as_pair_returns_pair() {
        assert_eq!(
            (Cell::from("D3"), Cell::from("G5")),
            CellSet::from("D3 G5").as_pair().unwrap()
        );
        assert_eq!(
            (Cell::from("F4"), Cell::from("J2")),
            CellSet::from("J2 F4").as_pair().unwrap()
        );
    }

    #[test]
    fn as_triple_returns_none_if_not_triple() {
        assert!(CellSet::empty().as_triple().is_none());
        assert!(CellSet::full().as_triple().is_none());
        assert!(CellSet::from("A5 D9 F3 H5").as_triple().is_none());
    }

    #[test]
    fn as_triple_returns_triple() {
        assert_eq!(
            (Cell::from("D3"), Cell::from("G5"), Cell::from("H2")),
            CellSet::from("D3 G5 H2").as_triple().unwrap()
        );
        assert_eq!(
            (Cell::from("E5"), Cell::from("F4"), Cell::from("J2")),
            CellSet::from("J2 F4 E5").as_triple().unwrap()
        );
    }

    #[test]
    fn from() {
        assert_eq!(CellSet::new(0b111), cells!("A1 A2 A3"));
        assert_eq!(CellSet::new(0b101010), cells!("A2 A4 A6"));
    }

    #[test]
    fn add_returns_the_same_set_when_the_cell_is_present() {
        let set = CellSet::new(0b10000001000001 as Bits);

        let got = set + "A7";
        assert_eq!(set, got);
    }

    #[test]
    fn add_returns_a_new_set_when_the_cell_is_not_present() {
        let set = CellSet::new(0b10000001000001 as Bits);

        let got = set + "G3";
        assert_ne!(set, got);
        assert!(got["G3"]);
    }

    #[test]
    fn sub_returns_the_same_set_when_the_cell_is_not_present() {
        let set = CellSet::new(0b10000001000001 as Bits);

        let got = set - "G3";
        assert_eq!(set, got);
    }

    #[test]
    fn sub_returns_the_same_set_when_the_cell_is_present() {
        let set = CellSet::new(0b10000001000001 as Bits);

        let got = set - "A7";
        assert_ne!(set, got);
        assert!(!got["A7"]);
    }

    #[test]
    fn not_returns_is_empty() {
        assert_eq!(true, !CellSet::empty());
        assert_eq!(false, !CellSet::full());
    }

    #[test]
    fn neg_returns_an_inverted_set() {
        assert_eq!(CellSet::full(), -CellSet::empty());
        assert_eq!(CellSet::empty(), -CellSet::full());

        assert_eq!(CellSet::full() - "A5" - "C9" - "G2", -cells!("A5 C9 G2"));
    }

    #[test]
    fn unions() {
        assert_eq!(CellSet::empty(), CellSet::empty() | CellSet::empty());
        assert_eq!(CellSet::full(), CellSet::full() | CellSet::empty());
        assert_eq!(CellSet::full(), CellSet::empty() | CellSet::full());
        assert_eq!(CellSet::full(), CellSet::full() | CellSet::full());

        assert_eq!(
            cells!("A5 B2 C9 D7 G2 J5"),
            cells!("A5 C9 G2") | cells!("B2 D7 J5")
        );
    }

    #[test]
    fn intersections() {
        assert_eq!(CellSet::empty(), CellSet::empty() & CellSet::empty());
        assert_eq!(CellSet::empty(), CellSet::full() & CellSet::empty());
        assert_eq!(CellSet::empty(), CellSet::empty() & CellSet::full());
        assert_eq!(CellSet::full(), CellSet::full() & CellSet::full());

        assert_eq!(
            cells!("A5 C9 G2"),
            cells!("A5 B6 C9 F3 G2 J2") & cells!("A5 B2 C9 D7 G2 J5")
        );
    }

    #[test]
    fn differences() {
        assert_eq!(CellSet::empty(), CellSet::empty() - CellSet::empty());
        assert_eq!(CellSet::full(), CellSet::full() - CellSet::empty());
        assert_eq!(CellSet::empty(), CellSet::empty() - CellSet::full());
        assert_eq!(CellSet::empty(), CellSet::full() - CellSet::full());

        let mut set = CellSet::full();
        set -= CellSet::full();
        assert!(set.is_empty());
    }

    #[test]
    fn strings() {
        let mut set = CellSet::empty();

        assert_eq!(EMPTY_SET, set.to_string());

        set += "C4";
        set += "B8";
        set += "F5";
        set += "H2";

        assert_eq!("B8 C4 F5 H2", set.to_string());
    }
}
