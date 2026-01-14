use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Neg};

use crate::layout::{Cell, CellSet, Coord, CoordSet};

use super::{CoordError, HouseSet, Iter, Shape};

/// One of the nine rows, columns, or blocks on the board.
#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq)]
pub struct House {
    shape: Shape,
    coord: Coord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HouseError {
    InvalidCoord { shape: Shape, error: CoordError },
}

impl fmt::Display for HouseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HouseError::InvalidCoord { shape, error } => {
                write!(f, "{} for {}", error, shape.code_label())
            }
        }
    }
}

impl std::error::Error for HouseError {}

impl House {
    pub const COUNT: u8 = 9;

    pub fn iter() -> HousesIter {
        HousesIter::new()
    }

    pub const fn all_rows() -> HouseSet {
        HouseSet::full(Shape::Row)
    }

    pub fn rows_iter() -> HouseIter {
        HouseIter::new(Shape::Row)
    }

    pub const fn row(coord: Coord) -> Self {
        Self::new(Shape::Row, coord)
    }

    pub const fn all_columns() -> HouseSet {
        HouseSet::full(Shape::Column)
    }

    pub fn columns_iter() -> HouseIter {
        HouseIter::new(Shape::Column)
    }

    pub const fn column(coord: Coord) -> Self {
        Self::new(Shape::Column, coord)
    }

    pub const fn all_blocks() -> HouseSet {
        HouseSet::full(Shape::Block)
    }

    pub fn blocks_iter() -> HouseIter {
        HouseIter::new(Shape::Block)
    }

    pub const fn block(coord: Coord) -> Self {
        Self::new(Shape::Block, coord)
    }

    pub fn try_from_with_shape(shape: Shape, label: &str) -> Result<Self, HouseError> {
        Ok(Self::new(
            shape,
            label
                .parse::<Coord>()
                .map_err(|error| HouseError::InvalidCoord { shape, error })?,
        ))
    }

    pub const fn new(shape: Shape, coord: Coord) -> Self {
        Self { shape, coord }
    }

    pub const fn shape(&self) -> Shape {
        self.shape
    }

    pub const fn is_row(&self) -> bool {
        self.shape.is_row()
    }

    pub const fn is_column(&self) -> bool {
        self.shape.is_column()
    }

    pub const fn is_block(&self) -> bool {
        self.shape.is_block()
    }

    pub const fn coord(&self) -> Coord {
        self.coord
    }

    pub const fn usize(&self) -> usize {
        self.coord.usize()
    }

    pub const fn label(&self) -> &str {
        LABELS[self.shape.usize()][self.coord.usize()]
    }

    pub const fn console_label(&self) -> char {
        CONSOLE_LABELS[self.shape.usize()][self.coord.usize()]
    }

    pub const fn is_top(&self) -> bool {
        self.is_row() && self.coord.u8() == 0
    }

    pub const fn is_block_top(&self) -> bool {
        self.is_row() && self.coord.u8().is_multiple_of(3)
    }

    pub const fn is_block_bottom(&self) -> bool {
        self.is_row() && self.coord.u8() % 3 == 2
    }

    pub const fn is_bottom(&self) -> bool {
        self.is_row() && self.coord.u8() == 8
    }

    pub const fn is_left(&self) -> bool {
        self.is_column() && self.coord.u8() == 0
    }

    pub const fn is_block_left(&self) -> bool {
        self.is_column() && self.coord.u8().is_multiple_of(3)
    }

    pub const fn is_block_right(&self) -> bool {
        self.is_column() && self.coord.u8() % 3 == 2
    }

    pub const fn is_right(&self) -> bool {
        self.is_column() && self.coord.u8() == 8
    }

    pub const fn cell(&self, coord: Coord) -> Cell {
        self.shape.cell(self.coord, coord)
    }

    pub const fn cells(&self) -> CellSet {
        self.shape.cells(self.coord)
    }

    pub const fn has(&self, cell: Cell) -> bool {
        self.cells().has(cell)
    }

    pub fn crossing_houses(&self, cells: CellSet) -> HouseSet {
        match self.shape() {
            Shape::Row => cells
                .iter()
                .fold(HouseSet::empty(Shape::Column), |acc, cell| {
                    acc + cell.column_coord()
                }),
            Shape::Column => cells.iter().fold(HouseSet::empty(Shape::Row), |acc, cell| {
                acc + cell.row_coord()
            }),
            Shape::Block => panic!("blocks do not have crossing houses"),
        }
    }

    pub fn intersect(&self, other: House) -> CellSet {
        INTERSECTIONS[self.shape.usize()][self.coord.usize()][other.shape.usize()]
            [other.coord.usize()]
    }

    pub const fn houses(&self, shape: Shape) -> HouseSet {
        match shape {
            Shape::Row => self.rows(),
            Shape::Column => self.columns(),
            Shape::Block => self.blocks(),
        }
    }

    pub const fn rows(&self) -> HouseSet {
        match self.shape {
            Shape::Row => ROW_ROWS[self.coord.usize()],
            Shape::Column => COLUMN_ROWS[self.coord.usize()],
            Shape::Block => BLOCK_ROWS[self.coord.usize()],
        }
    }

    pub fn row_iter(&self) -> Iter {
        self.rows().iter()
    }

    pub const fn columns(&self) -> HouseSet {
        match self.shape {
            Shape::Row => ROW_COLUMNS[self.coord.usize()],
            Shape::Column => COLUMN_COLUMNS[self.coord.usize()],
            Shape::Block => BLOCK_COLUMNS[self.coord.usize()],
        }
    }

    pub fn column_iter(&self) -> Iter {
        self.columns().iter()
    }

    pub const fn blocks(&self) -> HouseSet {
        match self.shape {
            Shape::Row => ROW_BLOCKS[self.coord.usize()],
            Shape::Column => COLUMN_BLOCKS[self.coord.usize()],
            Shape::Block => BLOCK_BLOCKS[self.coord.usize()],
        }
    }

    pub fn block_iter(&self) -> Iter {
        self.blocks().iter()
    }
}

impl PartialOrd<Self> for House {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.shape.partial_cmp(&other.shape) {
            Some(Ordering::Equal) => self.coord.partial_cmp(&other.coord),
            result => result,
        }
    }
}

impl Add<House> for House {
    type Output = HouseSet;

    fn add(self, rhs: House) -> HouseSet {
        HouseSet::empty(self.shape) + self + rhs
    }
}

impl Neg for House {
    type Output = HouseSet;

    fn neg(self) -> HouseSet {
        HouseSet::full(self.shape) - self
    }
}

impl fmt::Display for House {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

pub struct HouseIter {
    shape: Shape,
    coord: u8,
}

impl HouseIter {
    pub const fn new(shape: Shape) -> Self {
        Self { shape, coord: 0 }
    }
}

impl Iterator for HouseIter {
    type Item = House;

    fn next(&mut self) -> Option<Self::Item> {
        if self.coord == 9 {
            None
        } else {
            let house = House::new(self.shape, self.coord.into());
            self.coord += 1;
            Some(house)
        }
    }
}

impl ExactSizeIterator for HouseIter {
    fn len(&self) -> usize {
        9 - self.coord as usize
    }
}

pub struct HousesIter {
    shape: Shape,
    coord: u8,
}

impl HousesIter {
    pub const fn new() -> Self {
        Self {
            shape: Shape::Row,
            coord: 0,
        }
    }
}

impl Iterator for HousesIter {
    type Item = House;

    fn next(&mut self) -> Option<Self::Item> {
        if self.coord == 9 {
            match self.shape {
                Shape::Row => {
                    self.shape = Shape::Column;
                    self.coord = 0;
                }
                Shape::Column => {
                    self.shape = Shape::Block;
                    self.coord = 0;
                }
                Shape::Block => return None,
            }
        }
        let house = House::new(self.shape, self.coord.into());
        self.coord += 1;
        Some(house)
    }
}

impl ExactSizeIterator for HousesIter {
    fn len(&self) -> usize {
        match self.shape {
            Shape::Row => 18 + 9 - self.coord as usize,
            Shape::Column => 9 + 9 - self.coord as usize,
            Shape::Block => 9 - self.coord as usize,
        }
    }
}

/// Creates a row [`House`] from a coordinate label.
///
/// # Examples
///
/// ```
/// use sudoku_rust::row;
///
/// let r = row!(1);
/// let r = row!(A);
/// ```
///
/// # Panics
///
/// Panics if the coordinate is invalid.
#[macro_export]
macro_rules! row {
    ($coord:tt) => {
        match House::try_from_with_shape(Shape::Row, stringify!($coord)) {
            Ok(h) => h,
            Err(e) => panic!("row![{}]: {}", stringify!($coord), e),
        }
    };
}

/// Creates a column [`House`] from a coordinate label.
///
/// # Examples
///
/// ```
/// use sudoku_rust::col;
///
/// let c = col!(1);
/// let c = col!(A);
/// ```
///
/// # Panics
///
/// Panics if the coordinate is invalid.
#[macro_export]
macro_rules! col {
    ($coord:tt) => {
        match House::try_from_with_shape(Shape::Column, stringify!($coord)) {
            Ok(h) => h,
            Err(e) => panic!("col![{}]: {}", stringify!($coord), e),
        }
    };
}

/// Creates a block [`House`] from a coordinate label.
///
/// # Examples
///
/// ```
/// use sudoku_rust::block;
///
/// let b = block!(1);
/// let b = block!(5);
/// ```
///
/// # Panics
///
/// Panics if the coordinate is invalid.
#[macro_export]
macro_rules! block {
    ($coord:tt) => {
        match House::try_from_with_shape(Shape::Block, stringify!($coord)) {
            Ok(h) => h,
            Err(e) => panic!("block![{}]: {}", stringify!($coord), e),
        }
    };
}

#[rustfmt::skip]
pub const LABELS: [[&str; 9]; 3] = [
    ["Row A", "Row B", "Row C", "Row D", "Row E", "Row F", "Row G", "Row H", "Row J"],
    ["Col 1", "Col 2", "Col 3", "Col 4", "Col 5", "Col 6", "Col 7", "Col 8", "Col 9"],
    ["Box 1", "Box 2", "Box 3", "Box 4", "Box 5", "Box 6", "Box 7", "Box 8", "Box 9"],
];

#[rustfmt::skip]
pub const CONSOLE_LABELS: [[char; 9]; 3] = [
    ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J'],
    ['¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'],
    ['❶', '❷', '❸', '❹', '❺', '❻', '❼', '❽', '❾'],
];

#[rustfmt::skip]
pub const ALT_CONSOLE_LABELS: [[char; 9]; 3] = [
    ['Ⓐ', 'Ⓑ', 'Ⓒ', 'Ⓓ', 'Ⓔ', 'Ⓕ', 'Ⓖ', 'Ⓗ', 'Ⓙ'],
    ['①', '②', '③', '④', '⑤', '⑥', '⑦', '⑧', '⑨'],
    ['❶', '❷', '❸', '❹', '❺', '❻', '❼', '❽', '❾'],
];

pub const ROWS: [House; 9] = make_houses(Shape::Row);
pub const COLUMNS: [House; 9] = make_houses(Shape::Column);
pub const BLOCKS: [House; 9] = make_houses(Shape::Block);

const fn make_houses(shape: Shape) -> [House; 9] {
    let mut houses: [House; 9] = [House::new(Shape::Row, Coord::new(0)); 9];
    let mut i = 0;

    while i < 9 {
        houses[i] = House::new(shape, Coord::new(i as u8));
        i += 1;
    }
    houses
}

pub const INTERSECTIONS: [[[[CellSet; 9]; 3]; 9]; 3] = {
    let mut sets: [[[[CellSet; 9]; 3]; 9]; 3] = [[[[CellSet::empty(); 9]; 3]; 9]; 3];

    let mut i = 0;
    while i < 3 {
        let mut ii = 0;
        while ii < 9 {
            let mut j = 0;
            while j < 3 {
                let mut jj = 0;
                while jj < 9 {
                    sets[i][ii][j][jj] = House::new(Shape::new(i as u8), Coord::new(ii as u8))
                        .cells()
                        .intersect(House::new(Shape::new(j as u8), Coord::new(jj as u8)).cells());
                    jj += 1;
                }
                j += 1;
            }
            ii += 1;
        }
        i += 1;
    }
    sets
};

const ROW_ROWS: [HouseSet; 9] = [
    HouseSet::new(Shape::Row, CoordSet::from_digit(1)),
    HouseSet::new(Shape::Row, CoordSet::from_digit(2)),
    HouseSet::new(Shape::Row, CoordSet::from_digit(3)),
    HouseSet::new(Shape::Row, CoordSet::from_digit(4)),
    HouseSet::new(Shape::Row, CoordSet::from_digit(5)),
    HouseSet::new(Shape::Row, CoordSet::from_digit(6)),
    HouseSet::new(Shape::Row, CoordSet::from_digit(7)),
    HouseSet::new(Shape::Row, CoordSet::from_digit(8)),
    HouseSet::new(Shape::Row, CoordSet::from_digit(9)),
];

const COLUMN_ROWS: [HouseSet; 9] = [House::all_rows(); 9];

#[rustfmt::skip]
const BLOCK_ROWS: [HouseSet; 9] = [
    HouseSet::new(Shape::Row, CoordSet::new_triple(1, 2, 3)),
    HouseSet::new(Shape::Row, CoordSet::new_triple(1, 2, 3)),
    HouseSet::new(Shape::Row, CoordSet::new_triple(1, 2, 3)),
    HouseSet::new(Shape::Row, CoordSet::new_triple(4, 5, 6)),
    HouseSet::new(Shape::Row, CoordSet::new_triple(4, 5, 6)),
    HouseSet::new(Shape::Row, CoordSet::new_triple(4, 5, 6)),
    HouseSet::new(Shape::Row, CoordSet::new_triple(7, 8, 9)),
    HouseSet::new(Shape::Row, CoordSet::new_triple(7, 8, 9)),
    HouseSet::new(Shape::Row, CoordSet::new_triple(7, 8, 9)),
];

const ROW_COLUMNS: [HouseSet; 9] = [House::all_columns(); 9];

const COLUMN_COLUMNS: [HouseSet; 9] = [
    HouseSet::new(Shape::Column, CoordSet::from_digit(1)),
    HouseSet::new(Shape::Column, CoordSet::from_digit(2)),
    HouseSet::new(Shape::Column, CoordSet::from_digit(3)),
    HouseSet::new(Shape::Column, CoordSet::from_digit(4)),
    HouseSet::new(Shape::Column, CoordSet::from_digit(5)),
    HouseSet::new(Shape::Column, CoordSet::from_digit(6)),
    HouseSet::new(Shape::Column, CoordSet::from_digit(7)),
    HouseSet::new(Shape::Column, CoordSet::from_digit(8)),
    HouseSet::new(Shape::Column, CoordSet::from_digit(9)),
];

#[rustfmt::skip]
const BLOCK_COLUMNS: [HouseSet; 9] = [
    HouseSet::new(Shape::Column, CoordSet::new_triple(1, 2, 3)),
    HouseSet::new(Shape::Column, CoordSet::new_triple(4, 5, 6)),
    HouseSet::new(Shape::Column, CoordSet::new_triple(7, 8, 9)),
    HouseSet::new(Shape::Column, CoordSet::new_triple(1, 2, 3)),
    HouseSet::new(Shape::Column, CoordSet::new_triple(4, 5, 6)),
    HouseSet::new(Shape::Column, CoordSet::new_triple(7, 8, 9)),
    HouseSet::new(Shape::Column, CoordSet::new_triple(1, 2, 3)),
    HouseSet::new(Shape::Column, CoordSet::new_triple(4, 5, 6)),
    HouseSet::new(Shape::Column, CoordSet::new_triple(7, 8, 9)),
];

const ROW_BLOCKS: [HouseSet; 9] = [
    HouseSet::new(Shape::Block, CoordSet::new_triple(1, 2, 3)),
    HouseSet::new(Shape::Block, CoordSet::new_triple(1, 2, 3)),
    HouseSet::new(Shape::Block, CoordSet::new_triple(1, 2, 3)),
    HouseSet::new(Shape::Block, CoordSet::new_triple(4, 5, 6)),
    HouseSet::new(Shape::Block, CoordSet::new_triple(4, 5, 6)),
    HouseSet::new(Shape::Block, CoordSet::new_triple(4, 5, 6)),
    HouseSet::new(Shape::Block, CoordSet::new_triple(7, 8, 9)),
    HouseSet::new(Shape::Block, CoordSet::new_triple(7, 8, 9)),
    HouseSet::new(Shape::Block, CoordSet::new_triple(7, 8, 9)),
];

const COLUMN_BLOCKS: [HouseSet; 9] = [
    HouseSet::new(Shape::Block, CoordSet::new_triple(1, 4, 7)),
    HouseSet::new(Shape::Block, CoordSet::new_triple(1, 4, 7)),
    HouseSet::new(Shape::Block, CoordSet::new_triple(1, 4, 7)),
    HouseSet::new(Shape::Block, CoordSet::new_triple(2, 5, 8)),
    HouseSet::new(Shape::Block, CoordSet::new_triple(2, 5, 8)),
    HouseSet::new(Shape::Block, CoordSet::new_triple(2, 5, 8)),
    HouseSet::new(Shape::Block, CoordSet::new_triple(3, 6, 9)),
    HouseSet::new(Shape::Block, CoordSet::new_triple(3, 6, 9)),
    HouseSet::new(Shape::Block, CoordSet::new_triple(3, 6, 9)),
];

const BLOCK_BLOCKS: [HouseSet; 9] = [
    HouseSet::new(Shape::Block, CoordSet::from_digit(1)),
    HouseSet::new(Shape::Block, CoordSet::from_digit(2)),
    HouseSet::new(Shape::Block, CoordSet::from_digit(3)),
    HouseSet::new(Shape::Block, CoordSet::from_digit(4)),
    HouseSet::new(Shape::Block, CoordSet::from_digit(5)),
    HouseSet::new(Shape::Block, CoordSet::from_digit(6)),
    HouseSet::new(Shape::Block, CoordSet::from_digit(7)),
    HouseSet::new(Shape::Block, CoordSet::from_digit(8)),
    HouseSet::new(Shape::Block, CoordSet::from_digit(9)),
];

#[cfg(test)]
mod tests {
    use crate::*;

    use super::*;

    #[test]
    fn houses() {
        let house_sets = [House::all_rows(), House::all_columns(), House::all_blocks()];

        for houses in house_sets {
            let mut all = CellSet::empty();

            for (i, house) in houses.iter().enumerate() {
                assert_eq!(house, House::new(houses.shape(), Coord::new(i as u8)));
                assert_eq!(houses.shape(), house.shape());
                assert_eq!(Coord::new(i as u8), house.coord());
                assert_eq!(i, house.usize());
                if !matches!(houses.shape(), Shape::Row) {
                    assert_eq!(
                        format!("{} {}", houses.shape().label(), i + 1),
                        house.label()
                    );
                }

                let mut house_cells = CellSet::empty();
                (1..=9).for_each(|c| {
                    let cell = house.cell(c.into());
                    assert_eq!(house, cell.house(houses.shape()));
                    house_cells += cell
                });
                assert_eq!(house.cells(), house_cells);

                all |= house.cells();
            }

            assert_eq!(CellSet::full(), all);
        }
    }

    #[test]
    fn intersect() {
        assert_eq!(cells![A1 A2 A3], row!(A).intersect(block!(1)));
        assert_eq!(cells![A1 A2 A3], row!(A).intersect(block!(1)));
    }

    #[test]
    fn row_cells() {
        assert_eq!(cells![A1 A2 A3 A4 A5 A6 A7 A8 A9], row!(A).cells());
        assert_eq!(cells![B1 B2 B3 B4 B5 B6 B7 B8 B9], row!(B).cells());
        assert_eq!(cells![C1 C2 C3 C4 C5 C6 C7 C8 C9], row!(C).cells());
        assert_eq!(cells![D1 D2 D3 D4 D5 D6 D7 D8 D9], row!(D).cells());
        assert_eq!(cells![E1 E2 E3 E4 E5 E6 E7 E8 E9], row!(E).cells());
        assert_eq!(cells![F1 F2 F3 F4 F5 F6 F7 F8 F9], row!(F).cells());
        assert_eq!(cells![G1 G2 G3 G4 G5 G6 G7 G8 G9], row!(G).cells());
        assert_eq!(cells![H1 H2 H3 H4 H5 H6 H7 H8 H9], row!(H).cells());
        assert_eq!(cells![J1 J2 J3 J4 J5 J6 J7 J8 J9], row!(J).cells());
    }

    #[test]
    fn column_cells() {
        assert_eq!(cells![A1 B1 C1 D1 E1 F1 G1 H1 J1], col!(1).cells());
        assert_eq!(cells![A2 B2 C2 D2 E2 F2 G2 H2 J2], col!(2).cells());
        assert_eq!(cells![A3 B3 C3 D3 E3 F3 G3 H3 J3], col!(3).cells());
        assert_eq!(cells![A4 B4 C4 D4 E4 F4 G4 H4 J4], col!(4).cells());
        assert_eq!(cells![A5 B5 C5 D5 E5 F5 G5 H5 J5], col!(5).cells());
        assert_eq!(cells![A6 B6 C6 D6 E6 F6 G6 H6 J6], col!(6).cells());
        assert_eq!(cells![A7 B7 C7 D7 E7 F7 G7 H7 J7], col!(7).cells());
        assert_eq!(cells![A8 B8 C8 D8 E8 F8 G8 H8 J8], col!(8).cells());
        assert_eq!(cells![A9 B9 C9 D9 E9 F9 G9 H9 J9], col!(9).cells());
    }

    #[test]
    fn block_cells() {
        assert_eq!(cells![A1 A2 A3 B1 B2 B3 C1 C2 C3], block!(1).cells());
        assert_eq!(cells![A4 A5 A6 B4 B5 B6 C4 C5 C6], block!(2).cells());
        assert_eq!(cells![A7 A8 A9 B7 B8 B9 C7 C8 C9], block!(3).cells());
        assert_eq!(cells![D1 D2 D3 E1 E2 E3 F1 F2 F3], block!(4).cells());
        assert_eq!(cells![D4 D5 D6 E4 E5 E6 F4 F5 F6], block!(5).cells());
        assert_eq!(cells![D7 D8 D9 E7 E8 E9 F7 F8 F9], block!(6).cells());
        assert_eq!(cells![G1 G2 G3 H1 H2 H3 J1 J2 J3], block!(7).cells());
        assert_eq!(cells![G4 G5 G6 H4 H5 H6 J4 J5 J6], block!(8).cells());
        assert_eq!(cells![G7 G8 G9 H7 H8 H9 J7 J8 J9], block!(9).cells());
    }

    #[test]
    fn columns_cross_rows() {
        let main = row!(B);
        let cells = cells![B1 B2];

        assert_eq!(cols![1 2], main.crossing_houses(cells));
    }

    #[test]
    fn rows_cross_columns() {
        let main = col!(6);
        let cells = cells![C6 F6];

        assert_eq!(rows![C F], main.crossing_houses(cells));
    }
}
