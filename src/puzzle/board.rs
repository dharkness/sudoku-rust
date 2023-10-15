use std::fmt;

use crate::io::format_for_fancy_console;
use crate::layout::{Cell, CellSet, House, Known, KnownSet, Value};
use crate::solve::creates_deadly_rectangles;

use super::{Effects, Error, PseudoCell, Strategy};

/// Tracks the full state of a puzzle in play.
///
/// To allow solvers to run quickly, the state of the board
/// is stored in several forms, duplicating data to provide
/// performant read access at the cost of slower writes
/// and increased memory consumption.
///
/// The givens are cells that begin with a digit, the clues given
/// by the puzzle creator to make it solvable. When a cell
/// is either given as a clue or been solved with a digit,
/// it is called known.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Board {
    /// Cells that were given a digit at the start,
    /// often referred to as clues.
    givens: CellSet,

    /// All cells with a digit, both givens and solved cells.
    knowns: CellSet,

    /// Value of each cell, either a digit or unknown.
    values: [Value; 81],

    /// Set of available digits that may still be set for each cell.
    candidate_knowns_by_cell: [KnownSet; 81],

    /// Set of available cells for each digit.
    candidate_cells_by_known: [CellSet; 9],

    /// Every cell that has N candidates.
    cells_with_n_candidates: [CellSet; 10],

    /// Every cell solved or given for each digit.
    solved_cells_by_known: [CellSet; 9],
}

impl Board {
    /// Creates a new board with no givens and all cells unsolved.
    #[rustfmt::skip]
    pub const fn new() -> Board {
        Board {
            givens: CellSet::empty(),
            knowns: CellSet::empty(),
            values: [Value::unknown(); 81],
            candidate_knowns_by_cell: [KnownSet::full(); 81],
            candidate_cells_by_known: [CellSet::full(); 9],
            cells_with_n_candidates: [
                CellSet::empty(), CellSet::empty(), CellSet::empty(),
                CellSet::empty(), CellSet::empty(), CellSet::empty(),
                CellSet::empty(), CellSet::empty(), CellSet::empty(),
                CellSet::full(),
            ],
            solved_cells_by_known: [CellSet::empty(); 9],
        }
    }

    /// Returns true if the cell is unknown.
    pub const fn is_unknown(&self, cell: Cell) -> bool {
        !self.knowns.has(cell)
    }

    /// Returns the number of unknown cells in the puzzle.
    pub const fn unknown_count(&self) -> usize {
        81 - self.knowns.size()
    }

    /// Returns the set of all unknown cells.
    pub fn unknowns(&self) -> CellSet {
        -self.knowns
    }

    /// Returns an iterator of all unknown cells with their candidates.
    pub fn unknown_iter(&self) -> impl Iterator<Item = (Cell, KnownSet)> + '_ {
        self.unknowns()
            .into_iter()
            .map(|cell| (cell, self.candidates(cell)))
    }

    /// Returns true if the cell is known or a given.
    pub const fn is_known(&self, cell: Cell) -> bool {
        self.knowns.has(cell)
    }

    /// Returns the number of known cells in the puzzle, including givens.
    pub const fn known_count(&self) -> usize {
        self.knowns.size()
    }

    /// Returns the set of all known cells, including givens.
    pub const fn knowns(&self) -> CellSet {
        self.knowns
    }

    /// Returns an iterator of all known cells with their digit, including givens.
    pub fn known_iter(&self) -> impl Iterator<Item = (Cell, Known)> + '_ {
        self.knowns
            .into_iter()
            .map(|cell| (cell, self.value(cell).known().unwrap()))
    }

    /// Returns the set of digits to which any of the cells is set.
    pub fn all_knowns(&self, cells: CellSet) -> KnownSet {
        cells.iter().fold(KnownSet::empty(), |acc, cell| {
            self.value(cell).known().map_or(acc, |k| acc + k)
        })
    }

    /// Returns true if a cell in the house has the digit.
    pub fn is_house_known(&self, house: House, known: Known) -> bool {
        !(self.solved_cells_by_known[known.usize()] & house.cells()).is_empty()
    }

    /// Returns true if the cell is a given.
    pub const fn is_given(&self, cell: Cell) -> bool {
        self.givens.has(cell)
    }

    /// Returns the number of givens in the puzzle.
    pub const fn given_count(&self) -> usize {
        self.givens.size()
    }

    /// Returns the set of all givens.
    pub const fn givens(&self) -> CellSet {
        self.givens
    }

    /// Returns true if every cell on the board has a digit.
    pub const fn is_solved(&self) -> bool {
        self.knowns.is_full()
    }

    /// Returns the number of solved cells in the puzzle, excluding givens.
    pub const fn solved_count(&self) -> usize {
        self.knowns.size() - self.givens.size()
    }

    /// Returns the set of all solved cells, excluding givens.
    pub const fn solved(&self) -> CellSet {
        self.knowns.minus(self.givens)
    }

    /// Returns true if every cell in the house has a digit.
    pub fn is_house_solved(&self, house: House) -> bool {
        (-self.knowns & house.cells()).is_empty()
    }

    /// Returns the value of the cell, either a digit or unknown.
    pub const fn value(&self, cell: Cell) -> Value {
        self.values[cell.usize()]
    }

    /// Sets the cell to the candidate, marks it as a given,
    /// and returns true along with any follow-up actions found.
    ///
    /// See [`Board::set_known()`] for more details.
    pub fn set_given(&mut self, cell: Cell, known: Known, effects: &mut Effects) -> bool {
        if self.set_known(cell, known, effects) {
            self.givens += cell;
            true
        } else {
            false
        }
    }

    /// Sets the cell to the candidate and returns true
    /// along with any follow-up actions found.
    ///
    /// The candidate is removed from the cell's peers
    /// and its three houses, and the cell is removed
    /// as a candidate for all of its other candidates
    /// in its three houses.
    ///
    /// If any errors are caused while setting the cell,
    /// they are returned with the actions, and the puzzle
    /// will be left in an unsolvable state, but the internal
    /// state will be consistent.
    ///
    /// Returns false with no actions or errors
    /// if the known is not a candidate for the cell.
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

    /// Returns a new pseudo cell with the given cells and their candidates.
    pub fn pseudo_cell(&self, cells: CellSet) -> PseudoCell {
        PseudoCell::new(cells, self.all_candidates(cells))
    }

    /// Returns the set of combined candidates for the cells.
    pub fn all_candidates(&self, cells: CellSet) -> KnownSet {
        cells
            .iter()
            .fold(KnownSet::empty(), |acc, cell| acc | self.candidates(cell))
    }

    /// Returns the set of common candidates for the cells.
    pub fn common_candidates(&self, cells: CellSet) -> KnownSet {
        cells
            .iter()
            .fold(KnownSet::full(), |acc, cell| acc & self.candidates(cell))
    }

    /// Returns the set of candidates for the cell.
    pub const fn candidates(&self, cell: Cell) -> KnownSet {
        self.candidate_knowns_by_cell[cell.usize()]
    }

    /// Returns true if the cell has the candidate.
    pub const fn is_candidate(&self, cell: Cell, known: Known) -> bool {
        self.candidate_knowns_by_cell[cell.usize()].has(known)
    }

    /// Returns all cells that have N candidates.
    pub const fn cells_with_n_candidates(&self, n: usize) -> CellSet {
        debug_assert!(n <= 9);
        self.cells_with_n_candidates[n]
    }

    /// Returns an iterator of unknown cells with N candidates with their candidates.
    pub fn cell_candidates_with_n_candidates(
        &self,
        n: usize,
    ) -> impl Iterator<Item = (Cell, KnownSet)> + '_ {
        self.cells_with_n_candidates(n)
            .iter()
            .map(|cell| (cell, self.candidates(cell)))
    }

    /// Returns the set of cells that have the candidate.
    pub const fn candidate_cells(&self, known: Known) -> CellSet {
        self.candidate_cells_by_known[known.usize()]
    }

    /// Returns the set of cells in the house that have the candidate.
    pub fn house_candidate_cells(&self, house: House, known: Known) -> CellSet {
        house.cells() & self.candidate_cells(known)
    }

    /// Removes the candidate from the cell and returns true
    /// along with any follow-up actions found.
    ///
    /// The cell is removed as a candidate from its three houses.
    ///
    /// If any errors are caused while removing the candidate,
    /// they are returned with the actions, and the puzzle
    /// will be left in an unsolvable state, but the internal
    /// state will be consistent.
    ///
    /// Returns false with no actions or errors
    /// if the known is not a candidate for the cell.
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
        } else if let Some(single) = knowns.as_single() {
            effects.add_set(Strategy::NakedSingle, cell, single);
        }
        self.remove_candidate_cell_from_houses(cell, known, effects);

        true
    }

    /// Removes the cell as a candidate for the known
    /// from its three houses and returns true
    /// along with any follow-up actions found.
    ///
    /// If any errors are caused while removing the candidate,
    /// they are returned with the actions, and the puzzle
    /// will be left in an unsolvable state, but the internal
    /// state will be consistent.
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

    /// Removes the candidates from the cell and returns true
    /// along with any follow-up actions found.
    ///
    /// See [`Board::remove_candidate()`] for more details.
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

    /// Removes the candidate from the cells and returns true
    /// along with any follow-up actions found.
    ///
    /// See [`Board::remove_candidate()`] for more details.
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

    /// Removes the candidates from the cells and returns true
    /// along with any follow-up actions found.
    ///
    /// See [`Board::remove_candidate()`] for more details.
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

    /// Returns a new board with the digits of this board
    /// set as givens for the specified cells.
    ///
    /// If any specified cell is not known in this board,
    /// it is left unknown in the returned board.
    pub fn with_givens(&self, pattern: CellSet) -> (Board, Effects) {
        (pattern & self.knowns()).iter().fold(
            (Board::new(), Effects::new()),
            |(mut b, mut e), c| {
                b.set_given(c, self.value(c).known().unwrap(), &mut e);
                (b, e)
            },
        )
    }

    /// Returns a new board with the digits of this board
    /// except for the one in the given cell.
    pub fn without(&self, cell: Cell) -> (Board, Effects) {
        self.known_iter().filter(|(c, _)| *c != cell).fold(
            (Board::new(), Effects::new()),
            |(mut b, mut e), (c, k)| {
                b.set_given(c, k, &mut e);
                (b, e)
            },
        )
    }

    /// Returns the packed string format of the digits of this board
    /// with a period for each unknown cell and no spacing between rows.
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
