use super::{Block, Cell, CellSet, Column, Coord, Row};

/// Identifies a row, column or block.
pub enum House {
    Row(Row),
    Column(Column),
    Block(Block),
}

impl House {
    pub const fn coord(&self) -> Coord {
        match self {
            House::Row(r) => r.coord(),
            House::Column(c) => c.coord(),
            House::Block(b) => b.coord(),
        }
    }

    pub const fn cell(&self, coord: Coord) -> Cell {
        match self {
            House::Row(r) => r.cell(Column::new(coord)),
            House::Column(c) => c.cell(Row::new(coord)),
            House::Block(b) => b.cell(coord),
        }
    }

    pub fn cells(&self) -> CellSet {
        match self {
            House::Row(r) => r.cells(),
            House::Column(c) => c.cells(),
            House::Block(b) => b.cells(),
        }
    }
}

pub const ROWS: [House; 9] = [
    House::Row(Row::new(0)),
    House::Row(Row::new(1)),
    House::Row(Row::new(2)),
    House::Row(Row::new(3)),
    House::Row(Row::new(4)),
    House::Row(Row::new(5)),
    House::Row(Row::new(6)),
    House::Row(Row::new(7)),
    House::Row(Row::new(8)),
];
pub const COLUMNS: [House; 9] = [
    House::Column(Column::new(0)),
    House::Column(Column::new(1)),
    House::Column(Column::new(2)),
    House::Column(Column::new(3)),
    House::Column(Column::new(4)),
    House::Column(Column::new(5)),
    House::Column(Column::new(6)),
    House::Column(Column::new(7)),
    House::Column(Column::new(8)),
];
pub const BLOCKS: [House; 9] = [
    House::Block(Block::new(0)),
    House::Block(Block::new(1)),
    House::Block(Block::new(2)),
    House::Block(Block::new(3)),
    House::Block(Block::new(4)),
    House::Block(Block::new(5)),
    House::Block(Block::new(6)),
    House::Block(Block::new(7)),
    House::Block(Block::new(8)),
];
