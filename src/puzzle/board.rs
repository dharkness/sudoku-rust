use std::fmt;

use crate::layout::{Cell, CellSet, House, Known, KnownSet, Value};
use crate::solvers::deadly_rectangles::creates_deadly_rectangles;
use crate::symbols::UNKNOWN_VALUE;

use super::{Effects, Error, Strategy};

/// Tracks the full state of a puzzle in play.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Board {
    /// Cells that were solved at the start.
    givens: CellSet,
    /// Solved cells.
    knowns: CellSet,
    /// Values for all cells.
    values: [Value; 81],
    /// Knowns that are still possible for each cell.
    candidate_knowns: [KnownSet; 81],
    /// Cells that are still possible for each known.
    candidate_cells: [CellSet; 9],
    /// Cells that have N candidates.
    cells_with_n_candidates: [CellSet; 10],
    /// Cells that have been solved for each known.
    known_cells: [CellSet; 9],
}

impl Board {
    pub const fn new() -> Board {
        let mut board = Board {
            givens: CellSet::empty(),
            knowns: CellSet::empty(),
            values: [Value::unknown(); 81],
            candidate_knowns: [KnownSet::full(); 81],
            candidate_cells: [CellSet::full(); 9],
            cells_with_n_candidates: [CellSet::empty(); 10],
            known_cells: [CellSet::empty(); 9],
        };
        board.cells_with_n_candidates[9] = CellSet::full();
        board
    }

    pub fn given_count(&self) -> usize {
        self.givens.size()
    }

    pub fn is_given(&self, cell: Cell) -> bool {
        self.givens[cell]
    }

    pub fn known_count(&self) -> usize {
        self.knowns.size()
    }

    pub fn unknown_count(&self) -> usize {
        81 - self.knowns.size()
    }

    pub fn is_known(&self, cell: Cell) -> bool {
        self.knowns[cell]
    }

    pub fn known_iter(&self) -> impl Iterator<Item = (Cell, Known)> + '_ {
        self.knowns
            .into_iter()
            .map(|cell| (cell, self.value(cell).known()))
    }

    pub fn is_solved(&self) -> bool {
        self.knowns.is_full()
    }

    pub fn all_candidates(&self, cells: CellSet) -> KnownSet {
        cells
            .iter()
            .fold(KnownSet::empty(), |acc, cell| acc | self.candidates(cell))
    }

    pub fn common_candidates(&self, cells: CellSet) -> KnownSet {
        cells
            .iter()
            .fold(KnownSet::full(), |acc, cell| acc & self.candidates(cell))
    }

    pub fn candidates(&self, cell: Cell) -> KnownSet {
        self.candidate_knowns[cell.usize()]
    }

    pub fn is_candidate(&self, cell: Cell, known: Known) -> bool {
        self.candidate_knowns[cell.usize()][known]
    }

    pub fn cells_with_n_candidates(&self, n: usize) -> CellSet {
        self.cells_with_n_candidates[n]
    }

    pub fn cell_knowns_with_n_candidates(
        &self,
        n: usize,
    ) -> impl Iterator<Item = (Cell, KnownSet)> + '_ {
        self.cells_with_n_candidates(n)
            .iter()
            .map(|cell| (cell, self.candidates(cell)))
    }

    pub fn candidate_cells(&self, known: Known) -> CellSet {
        self.candidate_cells[known.usize()]
    }

    pub fn house_candidate_cells(&self, house: House, known: Known) -> CellSet {
        house.cells() & self.candidate_cells(known)
    }

    pub fn remove_candidate(&mut self, cell: Cell, known: Known, effects: &mut Effects) -> bool {
        let knowns = &mut self.candidate_knowns[cell.usize()];
        if knowns[known] {
            let size = knowns.size();
            self.cells_with_n_candidates[size] -= cell;
            self.cells_with_n_candidates[size - 1] += cell;
            *knowns -= known;
            if knowns.is_empty() {
                effects.add_error(Error::UnsolvableCell(cell));
            } else if knowns.size() == 1 {
                effects.add_set(Strategy::NakedSingle, cell, knowns.iter().next().unwrap());
            }

            let cells = &mut self.candidate_cells[known.usize()];
            debug_assert!(cells[cell]);
            *cells -= cell;
            self.remove_candidate_cell_from_houses(cell, known, effects);
            true
        } else {
            false
        }
    }

    fn remove_candidate_cell_from_houses(
        &mut self,
        cell: Cell,
        known: Known,
        effects: &mut Effects,
    ) {
        let all_candidates = self.candidate_cells[known.usize()];
        for house in cell.houses() {
            let house_cells = house.cells();
            if (self.known_cells[known.usize()] & house_cells).is_empty() {
                let candidates = all_candidates & house_cells;
                if candidates.is_empty() {
                    effects.add_error(Error::UnsolvableHouse(house, known));
                } else if candidates.size() == 1 {
                    effects.add_set(
                        Strategy::HiddenSingle,
                        candidates.iter().next().unwrap(),
                        known,
                    );
                }
            }
        }
    }

    pub fn remove_candidates(
        &mut self,
        cell: Cell,
        knowns: KnownSet,
        effects: &mut Effects,
    ) -> bool {
        knowns.iter().fold(false, |acc, known| {
            self.remove_candidate(cell, known, effects) || acc
        })
    }

    pub fn remove_candidate_from_cells(
        &mut self,
        cells: CellSet,
        known: Known,
        effects: &mut Effects,
    ) -> bool {
        cells.iter().fold(false, |acc, cell| {
            self.remove_candidate(cell, known, effects) || acc
        })
    }

    pub fn remove_candidates_from_cells(
        &mut self,
        cells: CellSet,
        knowns: KnownSet,
        effects: &mut Effects,
    ) -> bool {
        cells.iter().fold(false, |acc, cell| {
            knowns.iter().fold(false, |acc, known| {
                self.remove_candidate(cell, known, effects) || acc
            }) || acc
        })
    }

    pub fn value(&self, cell: Cell) -> Value {
        self.values[cell.usize()]
    }

    pub fn set_given(&mut self, cell: Cell, known: Known, effects: &mut Effects) -> bool {
        if self.set_known(cell, known, effects) {
            self.givens += cell;
            true
        } else {
            false
        }
    }

    pub fn set_known(&mut self, cell: Cell, known: Known, effects: &mut Effects) -> bool {
        if !self.is_candidate(cell, known) || self.is_known(cell) {
            return false;
        }

        if let Some(rectangles) = creates_deadly_rectangles(self, cell, known) {
            rectangles.into_iter().for_each(|r| {
                effects.add_error(Error::DeadlyRectangle(r));
            });
        }

        self.values[cell.usize()] = known.value();
        self.knowns += cell;
        self.known_cells[known.usize()] += cell;
        self.candidate_cells[known.usize()] -= cell;

        let mut candidates = self.candidate_knowns[cell.usize()];
        self.cells_with_n_candidates[candidates.size()] -= cell;
        self.cells_with_n_candidates[0] += cell;
        candidates -= known;
        self.candidate_knowns[cell.usize()] = KnownSet::empty();
        for known in candidates.iter() {
            self.candidate_cells[known.usize()] -= cell;
            self.remove_candidate_cell_from_houses(cell, known, effects);
        }

        for peer in (self.candidate_cells[known.usize()] & cell.peers()).iter() {
            self.remove_candidate(peer, known, effects);
            // effects.add_erase(Strategy::Peer, peer, known);
        }

        true
    }

    pub fn packed_string(&self, unknown: char) -> String {
        let mut result = String::new();
        House::rows_iter().for_each(|row| {
            result += " ";
            row.cells().iter().for_each(|cell| {
                let value = self.value(cell);
                if !value {
                    result.push(unknown);
                } else {
                    result.push(value.label());
                }
            })
        });
        result[1..].to_string()
    }

    pub fn fancy_string(&self) -> String {
        self.packed_string(UNKNOWN_VALUE)
    }

    pub fn console_string(&self) -> String {
        self.packed_string('.')
    }

    pub fn url_string(&self) -> String {
        self.packed_string('0')
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.fancy_string())
    }
}
