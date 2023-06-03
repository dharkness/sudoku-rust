use crate::layout::{Cell, CellSet, Coord};
use std::fmt;

/// The three house shapes on the board.
#[derive(Clone, Copy, Default, Eq, PartialEq, Ord, PartialOrd)]
pub enum Shape {
    #[default]
    Row,
    Column,
    Block,
}

impl Shape {
    pub const fn new(index: u8) -> Self {
        debug_assert!(index <= 2);
        match index {
            0 => Self::Row,
            1 => Self::Column,
            2 => Self::Block,
            _ => unreachable!(),
        }
    }

    pub const fn usize(&self) -> usize {
        *self as usize
    }

    pub const fn label(&self) -> &str {
        match self {
            Shape::Row => "Row",
            Shape::Column => "Col",
            Shape::Block => "Box",
        }
    }

    pub const fn cells(&self, house: Coord) -> CellSet {
        CELL_SETS[self.usize()][house.usize()]
    }

    pub const fn cell(&self, house: Coord, coord: Coord) -> Cell {
        CELLS[self.usize()][house.usize()][coord.usize()]
    }

    pub const fn cell_list(&self, house: Coord) -> &'static [Cell; 9] {
        &CELLS[self.usize()][house.usize()]
    }
}

impl From<char> for Shape {
    fn from(index: char) -> Self {
        match index {
            'R' => Shape::Row,
            'C' => Shape::Column,
            'B' => Shape::Block,
            _ => panic!("Invalid shape type: {}", index),
        }
    }
}

impl fmt::Display for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl fmt::Debug for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

const CELLS: [[[Cell; 9]; 9]; 3] = {
    let mut cells: [[[Cell; 9]; 9]; 3] = [[[Cell::new(0); 9]; 9]; 3];

    const fn shape_cells(shape: Shape) -> [[Cell; 9]; 9] {
        let mut cells: [[Cell; 9]; 9] = [[Cell::new(0); 9]; 9];
        let mut house = 0;

        while house < 9 {
            cells[house] = house_cells(shape, Coord::new(house as u8));
            house += 1;
        }
        cells
    }

    const fn house_cells(shape: Shape, house: Coord) -> [Cell; 9] {
        let mut cells: [Cell; 9] = [Cell::new(0); 9];
        let mut coord = 0;

        while coord < 9 {
            cells[coord] = house_cell(shape, house, Coord::new(coord as u8));
            coord += 1;
        }
        cells
    }

    const fn house_cell(shape: Shape, house: Coord, coord: Coord) -> Cell {
        match shape {
            Shape::Row => Cell::new(9 * house.u8() + coord.u8()),
            Shape::Column => Cell::new(house.u8() + 9 * coord.u8()),
            Shape::Block => Cell::new(
                (house.u8() / 3) * 27
                    + (house.u8() % 3) * 3
                    + (coord.u8() / 3) * 9
                    + (coord.u8() % 3),
            ),
        }
    }

    cells[Shape::Row.usize()] = shape_cells(Shape::Row);
    cells[Shape::Column.usize()] = shape_cells(Shape::Column);
    cells[Shape::Block.usize()] = shape_cells(Shape::Block);
    cells
};

const CELL_SETS: [[CellSet; 9]; 3] = {
    let mut sets: [[CellSet; 9]; 3] = [[CellSet::empty(); 9]; 3];

    const fn cell_sets(shape: Shape) -> [CellSet; 9] {
        let mut cell_sets: [CellSet; 9] = [CellSet::empty(); 9];
        let mut house = 0;

        while house < 9 {
            cell_sets[house] = CellSet::of::<9>(&CELLS[shape.usize()][house]);
            house += 1;
        }
        cell_sets
    }

    sets[Shape::Row.usize()] = cell_sets(Shape::Row);
    sets[Shape::Column.usize()] = cell_sets(Shape::Column);
    sets[Shape::Block.usize()] = cell_sets(Shape::Block);
    sets
};
