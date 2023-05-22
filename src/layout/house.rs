use core::array::from_fn;
use std::cmp::Ordering;
use std::ops::Deref;

use super::{Cell, CellSet, Coord};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Shape {
    Row,
    Column,
    Block,
}

impl Shape {
    const fn cells(&self, house: Coord) -> [Cell; 9] {
        let mut cells: [Cell; 9] = [Cell::new(0); 9];
        let mut i = 0;

        while i < 9 {
            cells[i] = self.cell(house, Coord::new(i as u8));
            i += 1;
        }
        cells
    }

    pub const fn cell(&self, house: Coord, coord: Coord) -> Cell {
        match self {
            Shape::Row => Cell::new(9 * house.u32() + coord.u32()),
            Shape::Column => Cell::new(house.u32() + 9 * coord.u32()),
            Shape::Block => Cell::new((house.u32() / 3) * 27 + (house.u32() % 3) * 3 + (coord.u32() / 3) * 9 + (coord.u32() % 3)),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct House {
    shape: Shape,
    coord: Coord,
    cells: [Cell; 9],
    set: CellSet,
}

impl House {
    pub const fn row(coord: Coord) -> Self {
        ROWS[coord.usize()]
    }

    pub const fn column(coord: Coord) -> Self {
        COLUMNS[coord.usize()]
    }

    pub const fn block(coord: Coord) -> Self {
        BLOCKS[coord.usize()]
    }

    const fn new(shape: Shape, coord: Coord) -> Self {
        let cells = shape.cells(coord);
        let set: CellSet = CellSet::from::<9>(&cells);
        Self { shape, coord, cells, set }
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
        self.cells[coord.usize()]
    }

    pub const fn cells(&self) -> CellSet {
        self.set
    }

    pub fn intersect(&self, house: House) -> CellSet {
        self.set & house.set
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

pub const ROWS: [House; 9] = [
    House::new(Shape::Row, Coord::new(0)),
    House::new(Shape::Row, Coord::new(1)),
    House::new(Shape::Row, Coord::new(2)),
    House::new(Shape::Row, Coord::new(3)),
    House::new(Shape::Row, Coord::new(4)),
    House::new(Shape::Row, Coord::new(5)),
    House::new(Shape::Row, Coord::new(6)),
    House::new(Shape::Row, Coord::new(7)),
    House::new(Shape::Row, Coord::new(8)),
];
pub const COLUMNS: [House; 9] = [
    House::new(Shape::Column, Coord::new(0)),
    House::new(Shape::Column, Coord::new(1)),
    House::new(Shape::Column, Coord::new(2)),
    House::new(Shape::Column, Coord::new(3)),
    House::new(Shape::Column, Coord::new(4)),
    House::new(Shape::Column, Coord::new(5)),
    House::new(Shape::Column, Coord::new(6)),
    House::new(Shape::Column, Coord::new(7)),
    House::new(Shape::Column, Coord::new(8)),
];
pub const BLOCKS: [House; 9] = [
    House::new(Shape::Block, Coord::new(0)),
    House::new(Shape::Block, Coord::new(1)),
    House::new(Shape::Block, Coord::new(2)),
    House::new(Shape::Block, Coord::new(3)),
    House::new(Shape::Block, Coord::new(4)),
    House::new(Shape::Block, Coord::new(5)),
    House::new(Shape::Block, Coord::new(6)),
    House::new(Shape::Block, Coord::new(7)),
    House::new(Shape::Block, Coord::new(8)),
];

pub const HOUSES: [House; 27] = [
    House::new(Shape::Row, Coord::new(0)),
    House::new(Shape::Row, Coord::new(1)),
    House::new(Shape::Row, Coord::new(2)),
    House::new(Shape::Row, Coord::new(3)),
    House::new(Shape::Row, Coord::new(4)),
    House::new(Shape::Row, Coord::new(5)),
    House::new(Shape::Row, Coord::new(6)),
    House::new(Shape::Row, Coord::new(7)),
    House::new(Shape::Row, Coord::new(8)),
    House::new(Shape::Column, Coord::new(0)),
    House::new(Shape::Column, Coord::new(1)),
    House::new(Shape::Column, Coord::new(2)),
    House::new(Shape::Column, Coord::new(3)),
    House::new(Shape::Column, Coord::new(4)),
    House::new(Shape::Column, Coord::new(5)),
    House::new(Shape::Column, Coord::new(6)),
    House::new(Shape::Column, Coord::new(7)),
    House::new(Shape::Column, Coord::new(8)),
    House::new(Shape::Block, Coord::new(0)),
    House::new(Shape::Block, Coord::new(1)),
    House::new(Shape::Block, Coord::new(2)),
    House::new(Shape::Block, Coord::new(3)),
    House::new(Shape::Block, Coord::new(4)),
    House::new(Shape::Block, Coord::new(5)),
    House::new(Shape::Block, Coord::new(6)),
    House::new(Shape::Block, Coord::new(7)),
    House::new(Shape::Block, Coord::new(8)),
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
                    let cell = h.cell(Coord::new(c));
                    assert_eq!(h, &cell.house(shape));
                    house += cell
                });
                assert_eq!(h.cells(), house);

                all |= h.cells();
            }

            assert_eq!(CellSet::full(), all);
        }
    }
}
