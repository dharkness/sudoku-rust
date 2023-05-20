use super::cell_set::{self, assert_cell, cell_from_label, Cell, CellSet, ALL_CELLS};
use super::known_set::{self, assert_known, Known, KnownSet, Value, ALL_KNOWNS, UNKNOWN};
use crate::cell_set::{
    block_from_cell, cell_in_block, cell_in_column, cell_in_row, column_from_cell, row_from_cell,
    ALL_COORDS,
};

pub struct Board {
    givens: CellSet,
    knowns: CellSet,
    values: [Value; 81],
    candidates: [KnownSet; 81],
}

impl Board {
    pub fn new() -> Board {
        Board {
            givens: cell_set::empty(),
            knowns: cell_set::empty(),
            values: [UNKNOWN; 81],
            candidates: [known_set::full(); 81],
        }
    }

    pub fn given_count(&self) -> u8 {
        cell_set::size(self.givens)
    }

    pub fn is_given(&self, cell: Cell) -> bool {
        cell_set::has(self.givens, cell)
    }

    pub fn known_count(&self) -> u8 {
        cell_set::size(self.knowns)
    }

    pub fn is_known(&self, cell: Cell) -> bool {
        cell_set::has(self.knowns, cell)
    }

    pub fn is_solved(&self) -> bool {
        self.known_count() == 81
    }

    pub fn candidates(&self, cell: Cell) -> KnownSet {
        assert_cell(cell);
        self.candidates[cell as usize]
    }

    pub fn is_candidate(&self, cell: Cell, known: Known) -> bool {
        assert_cell(cell);
        assert_known(known);
        known_set::has(self.candidates(cell), known)
    }

    pub fn value(&self, cell: Cell) -> Value {
        assert_cell(cell);
        self.values[cell as usize]
    }

    pub fn set_given(&mut self, cell: Cell, known: Known) {
        self.set_known(cell, known);
        cell_set::add(&mut self.givens, cell);
    }

    pub fn set_known(&mut self, cell: Cell, known: Known) {
        assert_cell(cell);
        assert_known(known);
        assert!(!self.is_known(cell));
        assert!(self.is_candidate(cell, known));
        self.values[cell as usize] = known;
        self.candidates[cell as usize] = known_set::empty();
        for i in ALL_COORDS {
            known_set::remove(
                &mut self.candidates[cell_in_row(row_from_cell(cell), i) as usize],
                known,
            );
            known_set::remove(
                &mut self.candidates[cell_in_column(column_from_cell(cell), i) as usize],
                known,
            );
            known_set::remove(
                &mut self.candidates[cell_in_block(block_from_cell(cell), i) as usize],
                known,
            );
        }
        cell_set::add(&mut self.knowns, cell);
    }

    pub fn remove_candidate(&mut self, cell: Cell, known: Known) {
        assert_cell(cell);
        assert_known(known);
        assert!(self.is_candidate(cell, known));
        known_set::remove(&mut self.candidates[cell as usize], known);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use rand::prelude::*;

    #[test]
    fn new_returns_an_empty_board() {
        let board = Board::new();

        assert!(!board.is_solved());
        assert_eq!(board.known_count(), 0);
        for c in ALL_CELLS {
            assert!(!board.is_known(c));
            assert_eq!(board.value(c), UNKNOWN);
            for k in ALL_KNOWNS {
                assert!(board.is_candidate(c, k));
            }
        }
    }

    #[test]
    fn solve_cell() {
        let mut board = Board::new();
        let cell = cell_from_label("D3");

        board.set_known(cell, 5);
        assert!(!board.is_solved());
        assert!(board.is_known(cell));
        assert_eq!(board.value(cell), 5);
        assert!(!board.is_candidate(cell, 5));
    }
}
