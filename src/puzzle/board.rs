use std::fmt;

use crate::io::format_for_fancy_console;
use crate::layout::{Cell, CellSet, House, Known, KnownSet, Value};
use crate::solve::creates_deadly_rectangles;

use super::{Effects, Error, PseudoCell, Strategy};

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
    candidate_knowns_by_cell: [KnownSet; 81],
    /// Cells that are still possible for each known.
    candidate_cells_by_known: [CellSet; 9],
    /// Cells that have N candidates.
    cells_with_n_candidates: [CellSet; 10],
    /// Cells that have been given or solved for each known.
    solved_cells_by_known: [CellSet; 9],
}

impl Board {
    pub const fn new() -> Board {
        let mut board = Board {
            givens: CellSet::empty(),
            knowns: CellSet::empty(),
            values: [Value::unknown(); 81],
            candidate_knowns_by_cell: [KnownSet::full(); 81],
            candidate_cells_by_known: [CellSet::full(); 9],
            cells_with_n_candidates: [CellSet::empty(); 10],
            solved_cells_by_known: [CellSet::empty(); 9],
        };
        board.cells_with_n_candidates[9] = CellSet::full();
        board
    }

    pub fn with_options(&self) -> Board {
        *self
    }

    pub const fn given_count(&self) -> usize {
        self.givens.size()
    }

    pub const fn is_given(&self, cell: Cell) -> bool {
        self.givens.has(cell)
    }

    pub const fn givens(&self) -> CellSet {
        self.givens
    }

    pub const fn known_count(&self) -> usize {
        self.knowns.size()
    }

    pub const fn unknown_count(&self) -> usize {
        81 - self.knowns.size()
    }

    pub const fn is_known(&self, cell: Cell) -> bool {
        self.knowns.has(cell)
    }

    pub const fn knowns(&self) -> CellSet {
        self.knowns
    }

    pub fn unknowns(&self) -> CellSet {
        -self.knowns
    }

    pub fn unknown_iter(&self) -> impl Iterator<Item = (Cell, KnownSet)> + '_ {
        self.unknowns()
            .into_iter()
            .map(|cell| (cell, self.candidates(cell)))
    }

    pub const fn solved(&self) -> CellSet {
        self.knowns.minus(self.givens)
    }

    pub fn known_iter(&self) -> impl Iterator<Item = (Cell, Known)> + '_ {
        self.knowns
            .into_iter()
            .map(|cell| (cell, self.value(cell).known().unwrap()))
    }

    pub const fn is_solved(&self) -> bool {
        self.knowns.is_full()
    }

    pub fn is_house_known(&self, house: House, known: Known) -> bool {
        !(self.solved_cells_by_known[known.usize()] & house.cells()).is_empty()
    }

    pub fn pseudo_cell(&self, cells: CellSet) -> PseudoCell {
        PseudoCell::new(cells, self.all_candidates(cells))
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

    pub const fn candidates(&self, cell: Cell) -> KnownSet {
        self.candidate_knowns_by_cell[cell.usize()]
    }

    pub const fn is_candidate(&self, cell: Cell, known: Known) -> bool {
        self.candidate_knowns_by_cell[cell.usize()].has(known)
    }

    pub const fn cells_with_n_candidates(&self, n: usize) -> CellSet {
        debug_assert!(n <= 9);
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

    pub const fn candidate_cells(&self, known: Known) -> CellSet {
        self.candidate_cells_by_known[known.usize()]
    }

    pub fn house_candidate_cells(&self, house: House, known: Known) -> CellSet {
        house.cells() & self.candidate_cells(known)
    }

    pub fn remove_candidate(&mut self, cell: Cell, known: Known, effects: &mut Effects) -> bool {
        let knowns = &mut self.candidate_knowns_by_cell[cell.usize()];
        if !knowns[known] {
            return false;
        }

        let size = knowns.size();
        *knowns -= known;
        self.cells_with_n_candidates[size] -= cell;
        self.cells_with_n_candidates[size - 1] += cell;
        self.candidate_cells_by_known[known.usize()] -= cell;

        if knowns.is_empty() {
            effects.add_error(Error::UnsolvableCell(cell));
        } else if knowns.size() == 1 {
            let single = knowns.as_single().unwrap();
            effects.add_set(Strategy::NakedSingle, cell, single);
        }
        self.remove_candidate_cell_from_houses(cell, known, effects);

        true
    }

    fn remove_candidate_cell_from_houses(
        &mut self,
        cell: Cell,
        known: Known,
        effects: &mut Effects,
    ) {
        for house in cell.houses() {
            if self.is_house_known(house, known) {
                continue;
            }

            let candidates = self.house_candidate_cells(house, known);
            if candidates.is_empty() {
                effects.add_error(Error::UnsolvableHouse(house, known));
            } else if candidates.size() == 1 {
                let single = candidates.as_single().unwrap();
                effects.add_set(Strategy::HiddenSingle, single, known);
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

    pub const fn value(&self, cell: Cell) -> Value {
        self.values[cell.usize()]
    }

    pub fn all_knowns(&self, cells: CellSet) -> KnownSet {
        cells.iter().fold(KnownSet::empty(), |acc, cell| {
            self.value(cell).known().map_or(acc, |k| acc + k)
        })
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
        if self.values[cell.usize()] == known.value() {
            return false;
        }

        if let Some(rectangles) = creates_deadly_rectangles(self, cell, known) {
            rectangles.into_iter().for_each(|r| {
                effects.add_error(Error::DeadlyRectangle(r));
            });
        }

        self.values[cell.usize()] = known.value();
        self.knowns += cell;
        self.solved_cells_by_known[known.usize()] += cell;
        self.candidate_cells_by_known[known.usize()] -= cell;

        let mut candidates = self.candidate_knowns_by_cell[cell.usize()];
        self.cells_with_n_candidates[candidates.size()] -= cell;
        self.cells_with_n_candidates[0] += cell;
        candidates -= known;
        self.candidate_knowns_by_cell[cell.usize()] = KnownSet::empty();
        for known in candidates {
            self.candidate_cells_by_known[known.usize()] -= cell;
            self.remove_candidate_cell_from_houses(cell, known, effects);
        }

        for peer in self.candidate_cells_by_known[known.usize()] & cell.peers() {
            self.remove_candidate(peer, known, effects);
            // effects.add_erase(Strategy::Peer, peer, known)
        }

        true
    }

    pub fn with_givens(&self, pattern: CellSet) -> (Board, Effects) {
        (pattern & self.knowns()).iter().fold(
            (Board::new(), Effects::new()),
            |(mut b, mut e), c| {
                b.set_given(c, self.value(c).known().unwrap(), &mut e);
                (b, e)
            },
        )
    }

    pub fn without(&self, cell: Cell) -> (Board, Effects) {
        self.known_iter().filter(|(c, _)| *c != cell).fold(
            (Board::new(), Effects::new()),
            |(mut b, mut e), (c, k)| {
                b.set_given(c, k, &mut e);
                (b, e)
            },
        )
    }

    pub fn packed_string(&self) -> String {
        let mut result = String::new();
        House::rows_iter().for_each(|row| {
            row.cells().iter().for_each(|cell| {
                let value = self.value(cell);
                if !value {
                    result.push('.');
                } else {
                    result.push(value.label());
                }
            })
        });
        result
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format_for_fancy_console(self))
    }
}
