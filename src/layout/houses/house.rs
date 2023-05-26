use core::array::from_fn;
use std::cmp::Ordering;
use std::ops::Deref;

use crate::layout::{Cell, CellSet, Coord, Shape};

#[derive(Clone, Copy, Debug)]
pub struct House {
    shape: Shape,
    coord: Coord,
}

impl House {
    pub const ROWS: [House; 9] = House::make_houses(Shape::Row);
    pub const COLUMNS: [House; 9] = House::make_houses(Shape::Column);
    pub const BLOCKS: [House; 9] = House::make_houses(Shape::Block);

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
            houses[i] = House::ROWS[i];
            houses[i + 9] = House::COLUMNS[i];
            houses[i + 18] = House::BLOCKS[i];
            i += 1;
        }
        houses
    };

    pub const fn row(coord: Coord) -> Self {
        House::ROWS[coord.usize()]
    }

    pub const fn column(coord: Coord) -> Self {
        House::COLUMNS[coord.usize()]
    }

    pub const fn block(coord: Coord) -> Self {
        House::BLOCKS[coord.usize()]
    }

    const fn new(shape: Shape, coord: Coord) -> Self {
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

    pub fn rows(&self) -> Vec<&Self> {
        match self.shape {
            Shape::Row => vec![self],
            Shape::Column => vec![],
            Shape::Block => {
                let first = 3 * (self.coord.usize() / 3);
                vec![&House::ROWS[first], &House::ROWS[first + 1], &House::ROWS[first + 2]]
            },
        }
    }

    pub fn columns(&self) -> Vec<&Self> {
        match self.shape {
            Shape::Row => vec![],
            Shape::Column => vec![self],
            Shape::Block => {
                let first = 3 * (self.coord.usize() % 3);
                vec![&House::COLUMNS[first], &House::COLUMNS[first + 1], &House::COLUMNS[first + 2]]
            },
        }
    }

    pub fn blocks(&self) -> Vec<&Self> {
        match self.shape {
            Shape::Row => {
                let first = 3 * (self.coord.usize() / 3);
                vec![&House::BLOCKS[first], &House::BLOCKS[first + 1], &House::BLOCKS[first + 2]]
            },
            Shape::Column => {
                let first = self.coord.usize() / 3;
                vec![&House::BLOCKS[first], &House::BLOCKS[first + 3], &House::BLOCKS[first + 6]]
            },
            Shape::Block => vec![self],
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn houses() {
        let groups = [
            (Shape::Row, House::ROWS),
            (Shape::Column, House::COLUMNS),
            (Shape::Block, House::BLOCKS),
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
