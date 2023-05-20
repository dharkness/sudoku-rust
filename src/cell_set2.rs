// While the tuple struct is a thin wrapper (should be same memory storage),
// the fact that it's a struct means it cannot be passed by value without moving it.
//
// Or maybe not. References are about ownership--not pointers.

use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Deref, Index, Not, Sub, SubAssign,
};

type CellIndex = u32;

#[derive(Copy, Clone)]
struct Cell(CellIndex);

impl Deref for Cell {
    type Target = CellIndex;

    fn deref(&self) -> &CellIndex {
        &self.0
    }
}

// FACTOR Switch to storing the bit in a u128? Or add CellBit for that and make Cell a full struct?
// When do we need the index of a cell?
// - calculate the row/col/block
// - create a label for logging
impl Cell {
    pub const fn new(value: CellIndex) -> Cell {
        debug_assert!(value < 81);
        Cell(value)
    }

    const fn of(value: CellIndex) -> Cell {
        Cell(value)
    }

    const fn from_bit(bit: u128) -> Cell {
        debug_assert!(bit.trailing_zeros() < 81 && bit == 1 << bit.trailing_zeros());
        Cell(bit.trailing_zeros())
    }

    pub const fn value(&self) -> CellIndex {
        self.0
    }

    pub const fn bit(&self) -> u128 {
        1 << self.0
    }

    pub const fn row(&self) -> CellIndex {
        self.0 / 9
    }

    pub const fn column(&self) -> CellIndex {
        self.0 % 9
    }

    pub const fn block(&self) -> CellIndex {
        (self.row() / 3) * 3 + (self.column() / 3)
    }
}

type Size = u32;
type Cells = u128;
type SizeAndCells = u128;

const CELL_COUNT: Size = 81;
pub const ALL_CELLS: std::ops::Range<Size> = 0..CELL_COUNT;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CellSet(SizeAndCells);

const ALL_BITS_SET: Cells = (1 << CELL_COUNT) - 1;

const CELLS_MASK: Cells = (1 << CELL_COUNT) - 1;
const SIZE_SHIFT: u32 = 128 - 32;

const SIZE_BIT: Cells = 1 << (128 - 32);

const FULL: SizeAndCells = pack(ALL_BITS_SET, CELL_COUNT);

const fn pack(cells: Cells, size: Size) -> SizeAndCells {
    debug_assert!(cells <= CELLS_MASK);
    debug_assert!(size <= CELL_COUNT);
    (((size as Cells) << SIZE_SHIFT) + cells) as SizeAndCells
}

impl CellSet {
    pub const fn empty() -> CellSet {
        CellSet(0)
    }

    pub const fn full() -> CellSet {
        CellSet(FULL)
    }

    pub const fn new(cells: Cells) -> CellSet {
        CellSet(pack(cells, cells.count_ones() as Size))
    }

    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub const fn is_full(&self) -> bool {
        self.size() == CELL_COUNT
    }

    pub const fn size(&self) -> Size {
        (self.0 >> SIZE_SHIFT) as Size
    }

    const fn cells(&self) -> Cells {
        self.0 & CELLS_MASK
    }

    pub const fn has(&self, cell: Cell) -> bool {
        self.0 & cell.bit() != 0
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

impl Add<Cell> for CellSet {
    type Output = Self;

    fn add(self, rhs: Cell) -> CellSet {
        if self.has(rhs) {
            self
        } else {
            CellSet(self.0 + rhs.bit() + SIZE_BIT)
        }
    }
}

impl AddAssign<Cell> for CellSet {
    fn add_assign(&mut self, rhs: Cell) {
        if !self.has(rhs) {
            self.0 += rhs.bit() + SIZE_BIT
        }
    }
}

impl Sub<Cell> for CellSet {
    type Output = Self;

    fn sub(self, rhs: Cell) -> CellSet {
        if !self.has(rhs) {
            self
        } else {
            CellSet(self.0 - rhs.bit() - SIZE_BIT)
        }
    }
}

impl SubAssign<Cell> for CellSet {
    fn sub_assign(&mut self, rhs: Cell) {
        if self.has(rhs) {
            self.0 -= rhs.bit() + SIZE_BIT
        }
    }
}

impl Not for CellSet {
    type Output = Self;

    fn not(self) -> CellSet {
        match self.0 {
            0 => CellSet(FULL),
            FULL => CellSet(0),
            _ => CellSet::new(!self.0 & CELLS_MASK),
        }
    }
}

impl BitOr for CellSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> CellSet {
        if self == rhs {
            self
        } else {
            CellSet::new((self.0 | rhs.0) & CELLS_MASK)
        }
    }
}

impl BitOrAssign for CellSet {
    fn bitor_assign(&mut self, rhs: Self) {
        if self.0 != rhs.0 {
            *self = CellSet::new((self.0 | rhs.0) & CELLS_MASK)
        }
    }
}

impl BitAnd for CellSet {
    type Output = Self;

    fn bitand(self, rhs: Self) -> CellSet {
        if self == rhs {
            self
        } else {
            CellSet::new((self.0 & rhs.0) & CELLS_MASK)
        }
    }
}

impl BitAndAssign for CellSet {
    fn bitand_assign(&mut self, rhs: Self) {
        if self.0 != rhs.0 {
            *self = CellSet::new((self.0 & rhs.0) & CELLS_MASK)
        }
    }
}

impl Sub for CellSet {
    type Output = Self;

    fn sub(self, rhs: Self) -> CellSet {
        CellSet::new((self.0 & !rhs.0) & CELLS_MASK)
    }
}

impl SubAssign for CellSet {
    fn sub_assign(&mut self, rhs: Self) {
        *self = CellSet::new((self.0 & !rhs.0) & CELLS_MASK)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_an_empty_set() {
        let set = CellSet::empty();

        assert!(set.is_empty());
        assert_eq!(0, set.size());
        for i in ALL_CELLS {
            assert!(!set.has(Cell(i)));
        }
    }

    #[test]
    fn full_returns_a_full_set() {
        let set = CellSet::full();

        assert!(!set.is_empty());
        assert_eq!(CELL_COUNT, set.size());
        for i in ALL_CELLS {
            assert!(set.has(Cell(i)));
        }
    }

    #[test]
    fn new_returns_a_set_with_the_given_bits() {
        let set = CellSet::new(
            0b101010101010101010101010101010101010101010101010101010101010101010101010101010101,
        );

        assert!(!set.is_empty());
        assert_eq!(41, set.size());
        for i in ALL_CELLS {
            assert_eq!(i % 2 == 0, set.has(Cell(i)));
        }
    }

    #[test]
    fn add_returns_the_same_set_when_the_cell_is_present() {
        let set = CellSet::new(0b10000001000001 as Cells);

        let got = set + Cell(6);
        assert_eq!(set, got);
    }

    #[test]
    fn add_returns_a_new_set_when_the_cell_is_not_present() {
        let set = CellSet::new(0b10000001000001 as Cells);

        let got = set + Cell(42);
        assert_ne!(set, got);
        assert!(got.has(Cell(42)));
    }

    #[test]
    fn sub_returns_the_same_set_when_the_cell_is_not_present() {
        let set = CellSet::new(0b10000001000001 as Cells);

        let got = set - Cell(42);
        assert_eq!(set, got);
    }

    #[test]
    fn sub_returns_the_same_set_when_the_cell_is_present() {
        let set = CellSet::new(0b10000001000001 as Cells);

        let got = set - Cell(6);
        assert_ne!(set, got);
        assert!(!got.has(Cell(6)));
    }

    #[test]
    fn not_returns_an_inverted_set() {
        assert_eq!(CellSet::full(), !CellSet::empty());
        assert_eq!(CellSet::empty(), !CellSet::full());

        assert_eq!(
            CellSet::new(
                0b100101010101010101010101101001011010100110101010101010101101010101010011001010101
            ),
            !CellSet::new(
                0b011010101010101010101010010110100101011001010101010101010010101010101100110101010
            )
        );
        assert_eq!(
            CellSet::new(
                0b011010101010101010101010010110100101011001010101010101010010101010101100110101010
            ),
            !CellSet::new(
                0b100101010101010101010101101001011010100110101010101010101101010101010011001010101
            )
        );
    }

    #[test]
    fn unions() {
        assert_eq!(CellSet::empty(), CellSet::empty() | CellSet::empty());
        assert_eq!(CellSet::full(), CellSet::full() | CellSet::empty());
        assert_eq!(CellSet::full(), CellSet::empty() | CellSet::full());
        assert_eq!(CellSet::full(), CellSet::full() | CellSet::full());

        let mut set = CellSet::empty();
        set |= CellSet::full();
        assert!(set.is_full());
    }

    #[test]
    fn intersections() {
        assert_eq!(CellSet::empty(), CellSet::empty() & CellSet::empty());
        assert_eq!(CellSet::empty(), CellSet::full() & CellSet::empty());
        assert_eq!(CellSet::empty(), CellSet::empty() & CellSet::full());
        assert_eq!(CellSet::full(), CellSet::full() & CellSet::full());

        let mut set = CellSet::full();
        set &= CellSet::empty();
        assert!(set.is_empty());
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
}
