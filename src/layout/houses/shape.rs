use crate::layout::{Cell, CellSet, Coord, House};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub enum Shape {
    #[default]
    Row,
    Column,
    Block,
}

impl Shape {
    pub const fn cells(self, house: Coord) -> CellSet {
        CELL_SETS[self as usize][house.usize()]
    }

    pub const fn cell(self, house: Coord, coord: Coord) -> Cell {
        CELLS[self as usize][house.usize()][coord.usize()]
    }

    pub const fn cell_list(self, house: Coord) -> &'static [Cell; 9] {
        &CELLS[self as usize][house.usize()]
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
            Shape::Row => Cell::new(9 * house.u32() + coord.u32()),
            Shape::Column => Cell::new(house.u32() + 9 * coord.u32()),
            Shape::Block => Cell::new((house.u32() / 3) * 27 + (house.u32() % 3) * 3 + (coord.u32() / 3) * 9 + (coord.u32() % 3)),
        }
    }

    cells[Shape::Row as usize] = shape_cells(Shape::Row);
    cells[Shape::Column as usize] = shape_cells(Shape::Column);
    cells[Shape::Block as usize] = shape_cells(Shape::Block);
    cells
};

const CELL_SETS: [[CellSet; 9]; 3] = {
    let mut sets: [[CellSet; 9]; 3] = [[CellSet::empty(); 9]; 3];

    const fn cell_sets(shape: Shape) -> [CellSet; 9] {
        let mut cell_sets: [CellSet; 9] = [CellSet::empty(); 9];
        let mut house = 0;

        while house < 9 {
            cell_sets[house] = CellSet::from::<9>(&CELLS[shape as usize][house]);
            house += 1;
        }
        cell_sets
    }

    sets[Shape::Row as usize] = cell_sets(Shape::Row);
    sets[Shape::Column as usize] = cell_sets(Shape::Column);
    sets[Shape::Block as usize] = cell_sets(Shape::Block);
    sets
};
