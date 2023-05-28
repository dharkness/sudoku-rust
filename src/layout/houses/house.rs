use std::cmp::Ordering;

use crate::layout::{Cell, CellSet, Coord, Shape};

/// One of the nine rows, columns, or blocks on the board.
#[derive(Clone, Copy, Debug, Default)]
pub struct House {
    shape: Shape,
    coord: Coord,
}

impl House {
    pub const fn all_houses() -> &'static [House; 27] {
        &ALL
    }

    pub const fn all_rows() -> &'static [House; 9] {
        &ROWS
    }

    pub const fn row(coord: Coord) -> Self {
        ROWS[coord.usize()]
    }

    pub const fn all_columns() -> &'static [House; 9] {
        &COLUMNS
    }

    pub const fn column(coord: Coord) -> Self {
        COLUMNS[coord.usize()]
    }

    pub const fn all_blocks() -> &'static [House; 9] {
        &BLOCKS
    }

    pub const fn block(coord: Coord) -> Self {
        BLOCKS[coord.usize()]
    }

    // TODO Add explicit const default() so this can stay private
    pub const fn new(shape: Shape, coord: Coord) -> Self {
        Self { shape, coord }
    }

    pub const fn shape(&self) -> Shape {
        self.shape
    }

    pub const fn coord(&self) -> Coord {
        self.coord
    }

    pub const fn usize(&self) -> usize {
        self.coord.usize()
    }

    pub const fn cell(&self, coord: Coord) -> Cell {
        self.shape.cell(self.coord, coord)
    }

    pub const fn cells(&self) -> CellSet {
        self.shape.cells(self.coord)
    }

    pub fn intersect(&self, other: House) -> CellSet {
        self.cells() & other.cells()
    }

    pub const fn rows(&self) -> &[House] {
        match self.shape {
            Shape::Row => &ROW_ROWS[self.coord.usize()],
            Shape::Column => &COLUMN_ROWS[self.coord.usize()],
            Shape::Block => &BLOCK_ROWS[self.coord.usize()],
        }
    }

    pub const fn columns(&self) -> &[House] {
        match self.shape {
            Shape::Row => &ROW_COLUMNS[self.coord.usize()],
            Shape::Column => &COLUMN_COLUMNS[self.coord.usize()],
            Shape::Block => &BLOCK_COLUMNS[self.coord.usize()],
        }
    }

    pub const fn blocks(&self) -> &[House] {
        match self.shape {
            Shape::Row => &ROW_BLOCKS[self.coord.usize()],
            Shape::Column => &COLUMN_BLOCKS[self.coord.usize()],
            Shape::Block => &BLOCK_BLOCKS[self.coord.usize()],
        }
    }
}

impl PartialEq<Self> for House {
    fn eq(&self, other: &Self) -> bool {
        self.shape == other.shape && self.coord == other.coord
    }
}

impl Eq for House {}

impl PartialOrd<Self> for House {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.shape.partial_cmp(&other.shape) {
            Some(Ordering::Equal) => self.coord.partial_cmp(&other.coord),
            result => result,
        }
    }
}

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

pub const ALL: [House; 27] = {
    let mut houses: [House; 27] = [House::new(Shape::Row, Coord::new(0)); 27];
    let mut i = 0;

    while i < 9 {
        houses[i] = ROWS[i];
        houses[i + 9] = COLUMNS[i];
        houses[i + 18] = BLOCKS[i];
        i += 1;
    }
    houses
};

#[rustfmt::skip]
const ROW_ROWS: [[House; 1]; 9] = [
    [ROWS[0]], [ROWS[1]], [ROWS[2]], [ROWS[3]], [ROWS[4]], [ROWS[5]], [ROWS[6]], [ROWS[7]], [ROWS[8]],
];

const COLUMN_ROWS: [[House; 9]; 9] = [ROWS; 9];

#[rustfmt::skip]
const BLOCK_ROWS: [[House; 3]; 9] = [
    [ROWS[0], ROWS[1], ROWS[2]], [ROWS[0], ROWS[1], ROWS[2]], [ROWS[0], ROWS[1], ROWS[2]],
    [ROWS[3], ROWS[4], ROWS[5]], [ROWS[3], ROWS[4], ROWS[5]], [ROWS[3], ROWS[4], ROWS[5]],
    [ROWS[6], ROWS[7], ROWS[8]], [ROWS[6], ROWS[7], ROWS[8]], [ROWS[6], ROWS[7], ROWS[8]],
];

const ROW_COLUMNS: [[House; 9]; 9] = [COLUMNS; 9];

#[rustfmt::skip]
const COLUMN_COLUMNS: [[House; 1]; 9] = [
    [COLUMNS[0]], [COLUMNS[1]], [COLUMNS[2]], [COLUMNS[3]], [COLUMNS[4]], [COLUMNS[5]], [COLUMNS[6]], [COLUMNS[7]], [COLUMNS[8]],
];

#[rustfmt::skip]
const BLOCK_COLUMNS: [[House; 3]; 9] = [
    [COLUMNS[0], COLUMNS[1], COLUMNS[2]], [COLUMNS[3], COLUMNS[4], COLUMNS[5]], [COLUMNS[6], COLUMNS[7], COLUMNS[8]],
    [COLUMNS[0], COLUMNS[1], COLUMNS[2]], [COLUMNS[3], COLUMNS[4], COLUMNS[5]], [COLUMNS[6], COLUMNS[7], COLUMNS[8]],
    [COLUMNS[0], COLUMNS[1], COLUMNS[2]], [COLUMNS[3], COLUMNS[4], COLUMNS[5]], [COLUMNS[6], COLUMNS[7], COLUMNS[8]],
];

#[rustfmt::skip]
const ROW_BLOCKS: [[House; 3]; 9] = [
    [BLOCKS[0], BLOCKS[1], BLOCKS[2]],
    [BLOCKS[0], BLOCKS[1], BLOCKS[2]],
    [BLOCKS[0], BLOCKS[1], BLOCKS[2]],
    [BLOCKS[3], BLOCKS[4], BLOCKS[5]],
    [BLOCKS[3], BLOCKS[4], BLOCKS[5]],
    [BLOCKS[3], BLOCKS[4], BLOCKS[5]],
    [BLOCKS[6], BLOCKS[7], BLOCKS[8]],
    [BLOCKS[6], BLOCKS[7], BLOCKS[8]],
    [BLOCKS[6], BLOCKS[7], BLOCKS[8]],
];

#[rustfmt::skip]
const COLUMN_BLOCKS: [[House; 3]; 9] = [
    [BLOCKS[0], BLOCKS[3], BLOCKS[6]],
    [BLOCKS[0], BLOCKS[3], BLOCKS[6]],
    [BLOCKS[0], BLOCKS[3], BLOCKS[6]],
    [BLOCKS[1], BLOCKS[4], BLOCKS[7]],
    [BLOCKS[1], BLOCKS[4], BLOCKS[7]],
    [BLOCKS[1], BLOCKS[4], BLOCKS[7]],
    [BLOCKS[2], BLOCKS[5], BLOCKS[8]],
    [BLOCKS[2], BLOCKS[5], BLOCKS[8]],
    [BLOCKS[2], BLOCKS[5], BLOCKS[8]],
];

#[rustfmt::skip]
const BLOCK_BLOCKS: [[House; 1]; 9] = [
    [BLOCKS[0]], [BLOCKS[1]], [BLOCKS[2]], [BLOCKS[3]], [BLOCKS[4]], [BLOCKS[5]], [BLOCKS[6]], [BLOCKS[7]], [BLOCKS[8]],
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn houses() {
        let groups = [
            (Shape::Row, ROWS),
            (Shape::Column, COLUMNS),
            (Shape::Block, BLOCKS),
        ];

        for group in groups {
            let (shape, houses) = group;
            let mut all = CellSet::empty();

            for (i, h) in houses.iter().enumerate() {
                assert_eq!(shape, h.shape());
                assert_eq!(Coord::new(i as u8), h.coord());
                assert_eq!(i, h.usize());

                let mut house = CellSet::empty();
                (0..9).for_each(|c| {
                    let cell = h.cell(c.into());
                    assert_eq!(h, &cell.house(shape));
                    house += cell
                });
                assert_eq!(h.cells(), house);

                all |= h.cells();
            }

            assert_eq!(CellSet::full(), all);
        }
    }

    #[test]
    fn row_cells() {
        assert_eq!(
            "( A1 A2 A3 A4 A5 A6 A7 A8 A9 )",
            format!("{}", House::row(0.into()).cells())
        );
        assert_eq!(
            "( B1 B2 B3 B4 B5 B6 B7 B8 B9 )",
            format!("{}", House::row(1.into()).cells())
        );
        assert_eq!(
            "( C1 C2 C3 C4 C5 C6 C7 C8 C9 )",
            format!("{}", House::row(2.into()).cells())
        );
        assert_eq!(
            "( D1 D2 D3 D4 D5 D6 D7 D8 D9 )",
            format!("{}", House::row(3.into()).cells())
        );
        assert_eq!(
            "( E1 E2 E3 E4 E5 E6 E7 E8 E9 )",
            format!("{}", House::row(4.into()).cells())
        );
        assert_eq!(
            "( F1 F2 F3 F4 F5 F6 F7 F8 F9 )",
            format!("{}", House::row(5.into()).cells())
        );
        assert_eq!(
            "( G1 G2 G3 G4 G5 G6 G7 G8 G9 )",
            format!("{}", House::row(6.into()).cells())
        );
        assert_eq!(
            "( H1 H2 H3 H4 H5 H6 H7 H8 H9 )",
            format!("{}", House::row(7.into()).cells())
        );
        assert_eq!(
            "( J1 J2 J3 J4 J5 J6 J7 J8 J9 )",
            format!("{}", House::row(8.into()).cells())
        );
    }

    #[test]
    fn column_cells() {
        assert_eq!(
            "( A1 B1 C1 D1 E1 F1 G1 H1 J1 )",
            format!("{}", House::column(0.into()).cells())
        );
        assert_eq!(
            "( A2 B2 C2 D2 E2 F2 G2 H2 J2 )",
            format!("{}", House::column(1.into()).cells())
        );
        assert_eq!(
            "( A3 B3 C3 D3 E3 F3 G3 H3 J3 )",
            format!("{}", House::column(2.into()).cells())
        );
        assert_eq!(
            "( A4 B4 C4 D4 E4 F4 G4 H4 J4 )",
            format!("{}", House::column(3.into()).cells())
        );
        assert_eq!(
            "( A5 B5 C5 D5 E5 F5 G5 H5 J5 )",
            format!("{}", House::column(4.into()).cells())
        );
        assert_eq!(
            "( A6 B6 C6 D6 E6 F6 G6 H6 J6 )",
            format!("{}", House::column(5.into()).cells())
        );
        assert_eq!(
            "( A7 B7 C7 D7 E7 F7 G7 H7 J7 )",
            format!("{}", House::column(6.into()).cells())
        );
        assert_eq!(
            "( A8 B8 C8 D8 E8 F8 G8 H8 J8 )",
            format!("{}", House::column(7.into()).cells())
        );
        assert_eq!(
            "( A9 B9 C9 D9 E9 F9 G9 H9 J9 )",
            format!("{}", House::column(8.into()).cells())
        );
    }

    #[test]
    fn block_cells() {
        assert_eq!(
            "( A1 A2 A3 B1 B2 B3 C1 C2 C3 )",
            format!("{}", House::block(0.into()).cells())
        );
        assert_eq!(
            "( A4 A5 A6 B4 B5 B6 C4 C5 C6 )",
            format!("{}", House::block(1.into()).cells())
        );
        assert_eq!(
            "( A7 A8 A9 B7 B8 B9 C7 C8 C9 )",
            format!("{}", House::block(2.into()).cells())
        );
        assert_eq!(
            "( D1 D2 D3 E1 E2 E3 F1 F2 F3 )",
            format!("{}", House::block(3.into()).cells())
        );
        assert_eq!(
            "( D4 D5 D6 E4 E5 E6 F4 F5 F6 )",
            format!("{}", House::block(4.into()).cells())
        );
        assert_eq!(
            "( D7 D8 D9 E7 E8 E9 F7 F8 F9 )",
            format!("{}", House::block(5.into()).cells())
        );
        assert_eq!(
            "( G1 G2 G3 H1 H2 H3 J1 J2 J3 )",
            format!("{}", House::block(6.into()).cells())
        );
        assert_eq!(
            "( G4 G5 G6 H4 H5 H6 J4 J5 J6 )",
            format!("{}", House::block(7.into()).cells())
        );
        assert_eq!(
            "( G7 G8 G9 H7 H8 H9 J7 J8 J9 )",
            format!("{}", House::block(8.into()).cells())
        );
    }
}
