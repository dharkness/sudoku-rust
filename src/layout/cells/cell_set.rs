// While the tuple struct is a thin wrapper (should be same memory storage),
// the fact that it's a struct means it cannot be passed by value without moving it.
//
// Or maybe not. References are about ownership--not pointers.

use std::fmt;
use std::iter::FusedIterator;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Index, Not, Sub, SubAssign,
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
    /// Returns a new empty set.
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Returns a new full set.
    pub const fn full() -> Self {
        Self(ALL_SET)
    }

    /// Returns a new set from the raw bit field `bits`.
    const fn new(bits: Bits) -> Self {
        debug_assert!(bits <= ALL_SET);
        Self(bits)
    }

    /// Returns a new set containing the cells with a digit in the packed string `puzzle`.
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

    /// Returns a new set containing each cell in `cells`.
    pub const fn of<const N: usize>(cells: &[Cell; N]) -> Self {
        let mut bits: Bits = 0;
        let mut i = 0;

        while i < N {
            bits |= cells[i].bit().bit();
            i += 1;
        }
        CellSet::new(bits)
    }

    /// Returns true if this set is empty.
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Returns true if this set is full.
    pub const fn is_full(&self) -> bool {
        self.0 == ALL_SET
    }

    /// Returns the number of cells in this set.
    pub const fn len(&self) -> usize {
        self.0.count_ones() as usize
    }

    /// Returns the cells in this set as a raw bit field.
    const fn bits(&self) -> Bits {
        self.0
    }

    /// Returns true if `cell` is a member of this set.
    pub const fn has(&self, cell: Cell) -> bool {
        self.0 & cell.bit().bit() != 0
    }

    /// Returns true if at least one of the members of `set` is a member of this set.
    pub const fn has_any(&self, set: CellSet) -> bool {
        !self.intersect(set).is_empty()
    }

    /// Returns true if all of the members of `subset` are members of this set.
    pub const fn has_all(&self, subset: CellSet) -> bool {
        self.intersect(subset).0 == subset.0
    }

    /// Returns true if all of the members of this set are members of `superset`.
    pub const fn is_subset_of(&self, superset: CellSet) -> bool {
        self.intersect(superset).0 == self.0
    }

    /// Returns the single cell in this set.
    ///
    /// # Returns
    ///
    /// - `Some(cell)`: If this set has exactly one cell.
    /// - `None`: If this set has zero or more than one cell.
    pub const fn as_single(&self) -> Option<Cell> {
        if self.len() != 1 {
            None
        } else {
            Some(Cell::new(self.bits().trailing_zeros() as u8))
        }
    }

    /// Returns the two cells in this set as a tuple.
    ///
    /// # Returns
    ///
    /// - `Some((first, second))`: If this set has exactly two cells.
    /// - `None`: If this set has zero or more than two cells.
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

    /// Returns the three cells in this set as a tuple.
    ///
    /// # Returns
    ///
    /// - `Some((first, second, third))`: If this set has exactly three cells.
    /// - `None`: If this set has zero or more than three cells.
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

    /// Returns a copy of this set with `cell` as a member.
    pub const fn with(&self, cell: Cell) -> Self {
        Self::new(self.0 | cell.bit().bit())
    }

    /// Adds `cell` to this set.
    pub fn add(&mut self, cell: Cell) {
        self.0 |= cell.bit().bit();
    }

    /// Returns a copy of this set without `cell` as a member.
    pub const fn without(&self, cell: Cell) -> Self {
        Self::new(self.0 & !(cell.bit().bit()))
    }

    /// Removes `cell` from this set.
    pub fn remove(&mut self, cell: Cell) {
        self.0 &= !(cell.bit().bit());
    }

    /// Returns the first cell in this set in row-then-column order.
    ///
    /// # Returns
    ///
    /// - `Some(cell)`: If this set has at least one cell.
    /// - `None`: If this set is empty.
    pub const fn first(&self) -> Option<Cell> {
        if self.is_empty() {
            None
        } else {
            Some(Cell::new(self.bits().trailing_zeros() as u8))
        }
    }

    /// Returns the first cell in this set in row-then-column order
    /// after removing it from this set.
    ///
    /// # Returns
    ///
    /// - `Some(cell)`: If this set has at least one cell.
    /// - `None`: If this set is empty.
    pub fn pop(&mut self) -> Option<Cell> {
        if self.is_empty() {
            None
        } else {
            let cell = Cell::new(self.bits().trailing_zeros() as u8);
            self.remove(cell);
            Some(cell)
        }
    }

    /// Returns a new set containing the combined members of this set and `set`.
    pub const fn union(&self, set: Self) -> Self {
        if self.0 == set.0 {
            *self
        } else {
            Self::new(self.0 | set.0)
        }
    }

    /// Adds the members of `set` to this set.
    pub fn union_with(&mut self, set: Self) {
        *self = self.union(set)
    }

    /// Returns a new set containing the common members of this set and `set`.
    pub const fn intersect(&self, set: Self) -> Self {
        if self.0 == set.0 {
            *self
        } else {
            Self::new(self.0 & set.0)
        }
    }

    /// Removes all members of this set that are not members of `set`.
    pub fn intersect_with(&mut self, set: Self) {
        *self = self.intersect(set)
    }

    /// Returns a new set containing the members of this set that are not in `set`.
    pub const fn minus(&self, set: Self) -> Self {
        if self.0 == set.0 {
            Self::empty()
        } else {
            Self::new(self.0 & !set.0)
        }
    }

    /// Removes all members of this set that are members of `set`.
    pub fn subtract(&mut self, set: Self) {
        *self = self.minus(set)
    }

    /// Returns a new set containing all cells that are not in this set.
    pub const fn inverted(&self) -> Self {
        Self::new(!self.0 & ALL_SET)
    }

    /// Removes all cells from this set and adds all other cells to it.
    pub fn invert(&mut self) {
        *self = self.inverted()
    }

    /// Returns the minimal set of rows containing the members of this set.
    pub fn rows(&self) -> HouseSet {
        self.houses(Shape::Row)
    }

    /// Returns the minimal set of columns containing the members of this set.
    pub fn columns(&self) -> HouseSet {
        self.houses(Shape::Column)
    }

    /// Returns the minimal set of blocks containing the members of this set.
    pub fn blocks(&self) -> HouseSet {
        self.houses(Shape::Block)
    }

    /// Returns the minimal set of `shape` houses containing the members of this set.
    pub fn houses(&self, shape: Shape) -> HouseSet {
        self.iter()
            .fold(HouseSet::empty(shape), |set, cell| set + cell.house(shape))
    }

    /// Returns an iterator over the members of this set in row-then-column order.
    pub const fn iter(&self) -> CellIter {
        CellIter {
            iter: self.bit_iter(),
        }
    }

    /// Returns an iterator over the members of this set as bits in row-then-column order.
    pub const fn bit_iter(&self) -> BitIter {
        BitIter { bits: self.bits() }
    }

    /// Returns a packed pattern string with a `1` for each member of this set.
    pub fn pattern_string(&self) -> String {
        (0..Cell::COUNT)
            .map(|i| if self.has(Cell::new(i)) { '1' } else { '.' })
            .collect()
    }

    /// Returns the size and bits of this set as a debug string.
    pub fn debug(&self) -> String {
        format!(
            "{:02}:{:081b}",
            self.len(),
            self.bits().reverse_bits() >> (128 - 81)
        )
    }
}

impl From<House> for CellSet {
    /// Returns a set containing the cells in `house`.
    fn from(house: House) -> Self {
        house.cells()
    }
}

impl From<&str> for CellSet {
    /// Returns a set containing the cells in `labels` after splitting on space.
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

    /// Returns an iterator over the members of this set in row-then-column order.
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
    /// Collects the cells in `iter` into a set.
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
    /// Collects all members of the sets in `iter` into a set.
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
    /// Collects the common members of the sets in `iter` into a set.
    fn intersection(self) -> CellSet {
        self.fold(CellSet::full(), |acc, c| acc & c)
    }
}

impl FromIterator<Cell> for CellSet {
    /// Collects the cells in `iter` into a set.
    fn from_iter<I: IntoIterator<Item = Cell>>(iter: I) -> Self {
        let mut set = Self::empty();
        for cell in iter {
            set += cell;
        }
        set
    }
}

impl FromIterator<CellSet> for CellSet {
    /// Collects all members of the sets in `iter` into a set.
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

    /// Returns true if the cell represented by `bit` is a member of this set.
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

    /// Returns true if `cell` is a member of this set.
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

    /// Returns true if the cell represented by `label` is a member of this set.
    fn index(&self, label: &str) -> &bool {
        if self.has(Cell::from(label)) {
            &true
        } else {
            &false
        }
    }
}

impl Add<Bit> for CellSet {
    type Output = Self;

    /// Returns a copy of this set with `rhs` as a member.
    fn add(self, rhs: Bit) -> Self {
        self.with(rhs.cell())
    }
}

impl Add<Cell> for CellSet {
    type Output = Self;

    /// Returns a copy of this set with `rhs` as a member.
    fn add(self, rhs: Cell) -> Self {
        self.with(rhs)
    }
}

impl Add<&str> for CellSet {
    type Output = Self;

    /// Returns a copy of this set with `rhs` as a member.
    fn add(self, rhs: &str) -> Self {
        self.with(Cell::from(rhs))
    }
}

impl AddAssign<Bit> for CellSet {
    /// Adds `rhs` to this set.
    fn add_assign(&mut self, rhs: Bit) {
        self.add(rhs.cell())
    }
}

impl AddAssign<Cell> for CellSet {
    /// Adds `rhs` to this set.
    fn add_assign(&mut self, rhs: Cell) {
        self.add(rhs)
    }
}

impl AddAssign<&str> for CellSet {
    /// Adds `rhs` to this set.
    fn add_assign(&mut self, rhs: &str) {
        self.add(Cell::from(rhs))
    }
}

impl Sub<Bit> for CellSet {
    type Output = Self;

    /// Returns a copy of this set without `rhs` as a member.
    fn sub(self, rhs: Bit) -> Self {
        self.without(rhs.cell())
    }
}

impl Sub<Cell> for CellSet {
    type Output = Self;

    /// Returns a copy of this set without `rhs` as a member.
    fn sub(self, rhs: Cell) -> Self {
        self.without(rhs)
    }
}

impl Sub<&str> for CellSet {
    type Output = Self;

    /// Returns a copy of this set without `rhs` as a member.
    fn sub(self, rhs: &str) -> Self {
        self.without(Cell::from(rhs))
    }
}

impl SubAssign<Bit> for CellSet {
    /// Removes `rhs` from this set.
    fn sub_assign(&mut self, rhs: Bit) {
        self.remove(rhs.cell())
    }
}

impl SubAssign<Cell> for CellSet {
    /// Removes `rhs` from this set.
    fn sub_assign(&mut self, rhs: Cell) {
        self.remove(rhs)
    }
}

impl SubAssign<&str> for CellSet {
    /// Removes `rhs` from this set.
    fn sub_assign(&mut self, rhs: &str) {
        self.remove(Cell::from(rhs))
    }
}

impl Not for CellSet {
    type Output = Self;

    /// Returns a new set containing all cells that are not in this set.
    fn not(self) -> Self {
        self.inverted()
    }
}

impl BitOr for CellSet {
    type Output = Self;

    /// Returns a new set containing the combined members of this set and `rhs`.
    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

impl BitOrAssign for CellSet {
    /// Adds the members of `rhs` to this set.
    fn bitor_assign(&mut self, rhs: Self) {
        self.union_with(rhs)
    }
}

impl BitAnd for CellSet {
    type Output = Self;

    /// Returns a new set containing the common members of this set and `rhs`.
    fn bitand(self, rhs: Self) -> Self {
        self.intersect(rhs)
    }
}

impl BitAndAssign for CellSet {
    /// Removes all members of this set that are not members of `rhs`.
    fn bitand_assign(&mut self, rhs: Self) {
        self.intersect_with(rhs)
    }
}

impl Sub for CellSet {
    type Output = Self;

    /// Returns a new set containing the members of this set that are not in `rhs`.
    fn sub(self, rhs: Self) -> Self {
        self.minus(rhs)
    }
}

impl SubAssign for CellSet {
    /// Removes all members of this set that are members of `rhs`.
    fn sub_assign(&mut self, rhs: Self) {
        self.subtract(rhs)
    }
}

impl fmt::Display for CellSet {
    /// Returns a string containing the labels of the cells in this set separated by spaces.
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

/// Returns a new set from the given bits, cells, or labels.
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

// TODO Inline this into CellIter?
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
    use crate::layout::cells::cell::cell;
    use crate::layout::houses::house_set::houses;

    #[test]
    fn empty() {
        let set = CellSet::empty();

        assert!(set.is_empty());
        assert_eq!(0, set.len());
        for i in ALL_CELLS {
            assert!(!set[Cell::new(i)]);
        }
    }

    #[test]
    fn full() {
        let set = CellSet::full();

        assert!(!set.is_empty());
        assert_eq!(Cell::COUNT, set.len() as u8);
        for i in ALL_CELLS {
            assert!(set[Cell::new(i)]);
        }
    }

    #[test]
    fn new() {
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
    fn new_from_pattern() {
        let set = CellSet::new_from_pattern(
            "
                7..1....9
                .2.3..7..
                4.9......
                .6.8..2..
                .........
                .7...1.5.
                .....49..
                .46..5..2
                .1...68..
            ",
        );
        assert_eq!(
            CellSet::from("A1 A4 A9 B2 B4 B7 C1 C3 D2 D4 D7 F2 F6 F8 G6 G7 H2 H3 H6 H9 J2 J6 J7"),
            set
        );
    }

    #[test]
    fn of() {
        let set = CellSet::of(&[cell!("A4"), cell!("G7"), cell!("C2"), cell!("J6")]);
        assert_eq!(CellSet::from("A4 C2 G7 J6"), set);
    }

    #[test]
    fn is_empty() {
        assert_eq!(true, CellSet::empty().is_empty());
        assert_eq!(false, CellSet::full().is_empty());
        assert_eq!(false, cells!("A5 D9 F3 H5").is_empty());
    }

    #[test]
    fn is_full() {
        assert_eq!(false, CellSet::empty().is_full());
        assert_eq!(true, CellSet::full().is_full());
        assert_eq!(false, cells!("A5 D9 F3 H5").is_full());
    }

    #[test]
    fn len() {
        assert_eq!(0, CellSet::empty().len());
        assert_eq!(81, CellSet::full().len());
        assert_eq!(4, cells!("A5 D9 F3 H5").len());
    }

    #[test]
    fn has() {
        assert_eq!(false, CellSet::empty().has(cell!("D4")));
        assert_eq!(true, CellSet::full().has(cell!("D4")));
        assert_eq!(false, cells!("A5 D9 F3 H5").has(cell!("E8")));
        assert_eq!(true, cells!("A5 D9 F3 H5").has(cell!("F3")));
    }

    #[test]
    fn has_any() {
        let set = cells!("A5 D9 F3 H5");

        assert_eq!(false, CellSet::empty().has_any(set));
        assert_eq!(true, CellSet::full().has_any(set));
        assert_eq!(true, set.has_any(set));
        assert_eq!(false, set.has_any(cells!("B8 D3")));
        assert_eq!(true, set.has_any(cells!("A5 F3")));
        assert_eq!(true, set.has_any(cells!("A5 B8 D3")));
    }

    #[test]
    fn has_all() {
        let set = cells!("A5 D9 F3 H5");

        assert_eq!(false, CellSet::empty().has_all(set));
        assert_eq!(true, CellSet::full().has_all(set));
        assert_eq!(true, set.has_all(set));
        assert_eq!(true, set.has_all(cells!("D9 H5")));
        assert_eq!(false, set.has_all(cells!("A5 B8 D3")));
    }

    #[test]
    fn is_subset_of() {
        let set = cells!("A5 D9 F3 H5");

        assert_eq!(false, set.is_subset_of(CellSet::empty()));
        assert_eq!(true, set.is_subset_of(CellSet::full()));
        assert_eq!(true, set.is_subset_of(set));
        assert_eq!(true, cells!("D9 H5").is_subset_of(set));
        assert_eq!(false, cells!("A5 C2 F3").is_subset_of(set));
    }

    #[test]
    fn as_single_returns_none_if_not_single() {
        assert!(CellSet::empty().as_single().is_none());
        assert!(CellSet::full().as_single().is_none());
        assert!(cells!("A5 D9 F3 H5").as_single().is_none());
    }

    #[test]
    fn as_single_returns_single() {
        assert_eq!(cell!("D3"), cells!("D3").as_single().unwrap());
        assert_eq!(cell!("F4"), cells!("F4").as_single().unwrap());
    }

    #[test]
    fn as_pair_returns_none_if_not_pair() {
        assert!(CellSet::empty().as_pair().is_none());
        assert!(CellSet::full().as_pair().is_none());
        assert!(cells!("A5 D9 F3 H5").as_pair().is_none());
    }

    #[test]
    fn as_pair_returns_pair() {
        assert_eq!(
            (cell!("D3"), cell!("G5")),
            cells!("D3 G5").as_pair().unwrap()
        );
        assert_eq!(
            (cell!("F4"), cell!("J2")),
            cells!("J2 F4").as_pair().unwrap()
        );
    }

    #[test]
    fn as_triple_returns_none_if_not_triple() {
        assert!(CellSet::empty().as_triple().is_none());
        assert!(CellSet::full().as_triple().is_none());
        assert!(cells!("A5 D9 F3 H5").as_triple().is_none());
    }

    #[test]
    fn as_triple_returns_triple() {
        assert_eq!(
            (cell!("D3"), cell!("G5"), cell!("H2")),
            cells!("D3 G5 H2").as_triple().unwrap()
        );
        assert_eq!(
            (cell!("E5"), cell!("F4"), cell!("J2")),
            cells!("J2 F4 E5").as_triple().unwrap()
        );
    }

    #[test]
    fn first_returns_none_if_empty() {
        assert_eq!(true, CellSet::empty().first().is_none());
    }

    #[test]
    fn first() {
        assert_eq!(cell!("D3"), cells!("D3 G5 H2").first().unwrap());
        assert_eq!(cell!("E5"), cells!("J2 F4 E5").first().unwrap());
    }

    #[test]
    fn pop_returns_none_if_empty() {
        let mut set = CellSet::empty();

        assert_eq!(true, set.pop().is_none());
        assert_eq!(true, set.is_empty());
    }

    #[test]
    fn pop() {
        let mut set = cells!("J2 F4 E5");

        assert_eq!(cell!("E5"), set.pop().unwrap());
        assert_eq!(cells!("F4 J2"), set);
    }

    #[test]
    fn intersect_with() {
        let mut set = cells!("A5 B8 D3");

        set.intersect_with(cells!("A5 D9 B8 J2"));
        assert_eq!(cells!("A5 B8"), set);
    }

    #[test]
    fn invert() {
        let mut set = cells!("A5 B8 D3");

        set.invert();
        assert_eq!(false, set.has(cell!("A5")));
        assert_eq!(false, set.has(cell!("B8")));
        assert_eq!(false, set.has(cell!("D3")));
        assert_eq!(true, set.has(cell!("J2")));
        assert_eq!(true, set.has(cell!("C7")));

        set += cell!("A5");
        set += cell!("B8");
        set += cell!("D3");
        assert_eq!(CellSet::full(), set)
    }

    #[test]
    fn rows() {
        assert_eq!(houses!("R1 R3 R7 R8"), cells!("A5 C2 C8 G9 H3 H6").rows());
    }

    #[test]
    fn columns() {
        assert_eq!(
            houses!("C2 C3 C5 C6 C8 C9"),
            cells!("A5 C2 C8 G9 H3 H6").columns()
        );
    }

    #[test]
    fn blocks() {
        assert_eq!(
            houses!("B1 B2 B3 B7 B8 B9"),
            cells!("A5 C2 C8 G9 H3 H6").blocks()
        );
    }

    #[test]
    fn from_house() {
        assert_eq!(
            cells!(House::from("R3")),
            cells!("C1 C2 C3 C4 C5 C6 C7 C8 C9")
        );
        assert_eq!(
            cells!(House::from("B3")),
            cells!("A7 A8 A9 B7 B8 B9 C7 C8 C9")
        );
    }

    #[test]
    fn from_labels() {
        assert_eq!(CellSet::new(0b111), cells!("A1 A2 A3"));
        assert_eq!(CellSet::new(0b101010), cells!("A2 A4 A6"));
    }

    #[test]
    fn from_cell_iterator() {
        let cells = cells!("A7 A8 A9 B7 B8 B9 C7 C8 C9");

        assert_eq!(
            cells!("A7 A8 A9 B7 B8 B9 C7 C8 C9"),
            CellSet::from_iter(cells.iter())
        );
    }

    #[test]
    fn from_cell_set_iterator() {
        let cells = vec![
            cells!("A7 A9 B8 C7 C9"),
            cells!("A7 A9 C7 C9"),
            cells!("A8 B7 B8 B9 C8"),
        ];

        assert_eq!(
            cells!("A7 A8 A9 B7 B8 B9 C7 C8 C9"),
            CellSet::from_iter(cells.iter().cloned())
        );
    }

    #[test]
    fn index_bit() {
        assert_eq!(true, cells!("A1 A2 A3")[Cell::from(0b10)]);
        assert_eq!(false, cells!("A1 A2 A3")[Cell::from(0b1000)]);
    }

    #[test]
    fn index_cell() {
        assert_eq!(true, cells!("A1 A2 A3")[cell!("A2")]);
        assert_eq!(false, cells!("A1 A2 A3")[cell!("C2")]);
    }

    #[test]
    fn index_label() {
        assert_eq!(true, cells!("A1 A2 A3")["A2"]);
        assert_eq!(false, cells!("A1 A2 A3")["C2"]);
    }

    #[test]
    fn add_returns_the_same_set_when_the_cell_is_present() {
        let set = cells!("B3 A7 H5");

        let got = set + "A7";
        assert_eq!(set, got);
    }

    #[test]
    fn add_returns_a_new_set_when_the_cell_is_not_present() {
        let set = cells!("B3 A7 H5");

        let got = set + "G3";
        assert_eq!(cells!("B3 A7 G3 H5"), got);
    }

    #[test]
    fn sub_returns_the_same_set_when_the_cell_is_not_present() {
        let set = cells!("B3 A7 H5");

        let got = set - "G3";
        assert_eq!(set, got);
    }

    #[test]
    fn sub_returns_the_same_set_when_the_cell_is_present() {
        let set = cells!("B3 A7 H5");

        let got = set - "A7";
        assert_eq!(cells!("B3 H5"), got);
    }

    #[test]
    fn inverted() {
        assert_eq!(CellSet::full(), !CellSet::empty());
        assert_eq!(CellSet::empty(), !CellSet::full());

        assert_eq!(CellSet::full() - "A5" - "C9" - "G2", !cells!("A5 C9 G2"));
    }

    #[test]
    fn union() {
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
    fn intersect() {
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
    fn minus() {
        assert_eq!(CellSet::empty(), CellSet::empty() - CellSet::empty());
        assert_eq!(CellSet::full(), CellSet::full() - CellSet::empty());
        assert_eq!(CellSet::empty(), CellSet::empty() - CellSet::full());
        assert_eq!(CellSet::empty(), CellSet::full() - CellSet::full());

        let mut set = CellSet::full();
        set -= CellSet::full();
        assert!(set.is_empty());
    }

    #[test]
    fn debug() {
        assert_eq!(
            "00:000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            CellSet::empty().debug()
        );
        assert_eq!(
            "04:000000000000000010000100000000000000000000000000010000000000000010000000000000000",
            cells!("B8 C4 F5 H2").debug()
        );
    }

    #[test]
    fn to_string() {
        assert_eq!(EMPTY_SET, CellSet::empty().to_string());
        assert_eq!("B8 C4 F5 H2", cells!("B8 C4 F5 H2").to_string());
    }

    #[test]
    fn fmt_debug() {
        assert_eq!(EMPTY_SET, format!("{:?}", CellSet::empty()));
        assert_eq!("B8 C4 F5 H2", format!("{:?}", cells!("B8 C4 F5 H2")));
    }

    #[test]
    fn pattern_string() {
        assert_eq!(
            ".................................................................................",
            CellSet::empty().pattern_string()
        );
        assert_eq!(
            "................1....1...........................1..............1................",
            cells!("B8 C4 F5 H2").pattern_string()
        );
    }
}
