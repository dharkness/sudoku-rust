use std::fmt;
use std::ops::{BitAnd, BitAndAssign};

use crate::io::format_for_fancy_console;
use crate::layout::{Cell, CellSet, Digit, DigitSet, House, PeerSet, Value};
use crate::solve::creates_deadly_rectangles;

use super::{Effects, Error, PseudoCell, Strategy};

/// Indicates the result of solving a cell or removing a candidate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Change {
    None,
    Valid,
    Invalid,
}

impl Change {
    pub fn changed(self) -> bool {
        self != Change::None
    }

    pub fn and(self, other: Change) -> Change {
        match (self, other) {
            (Change::None, _) => other,
            (_, Change::None) => self,
            (Change::Valid, Change::Valid) => Change::Valid,
            _ => Change::Invalid,
        }
    }
}

impl BitAnd for Change {
    type Output = Change;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.and(rhs)
    }
}

impl BitAndAssign for Change {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = self.and(rhs);
    }
}

/// Tracks the full state of a puzzle for strategy evaluation.
///
/// `Board` is a read-optimized snapshot with cached views over solved cells,
/// candidates per cell, and candidate cells per digit. These caches make
/// common strategy queries fast at the cost of slower writes and more memory.
///
/// Typical read patterns include:
/// - `candidates(cell)` and `is_candidate(cell, digit)`
/// - `candidate_cells(digit)` and `house_candidate_cells(house, digit)`
/// - `solved()`, `unsolved()`, `givens()`, `placed()`
/// - `cells_with_n_candidates(n)` and `combined_candidates(cells)`
///
/// Strategies should not mutate the board directly. Instead, create
/// `Action`s and return them in `Effects`. Mutations are applied by
/// `Action::apply` or `Changer`, and any errors are reported via `Effects`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Board {
    /// Cells that were given a digit at the start,
    /// often referred to as clues.
    givens: CellSet,

    /// Every solved (given or placed) cell.
    solved_cells: CellSet,

    /// Every solved (given or placed) cell for each digit.
    solved_cells_by_digit: [CellSet; 9],

    /// Value of each cell, either a digit or none.
    values: [Value; 81],

    /// Set of available digits that may still be set for each cell.
    candidate_digits_by_cell: [DigitSet; 81],

    /// Set of available cells for each digit.
    candidate_cells_by_digit: [CellSet; 9],

    /// Every cell that has N candidates.
    cells_with_n_candidates: [CellSet; 10],
}

impl Board {
    /// Creates a new board with no givens and all cells unsolved.
    #[rustfmt::skip]
    pub const fn new() -> Board {
        Board {
            givens: CellSet::empty(),
            solved_cells: CellSet::empty(),
            solved_cells_by_digit: [CellSet::empty(); 9],
            values: [Value::none(); 81],
            candidate_digits_by_cell: [DigitSet::full(); 81],
            candidate_cells_by_digit: [CellSet::full(); 9],
            cells_with_n_candidates: [
                CellSet::empty(), CellSet::empty(), CellSet::empty(),
                CellSet::empty(), CellSet::empty(), CellSet::empty(),
                CellSet::empty(), CellSet::empty(), CellSet::empty(),
                CellSet::full(),
            ],
        }
    }

    /// Returns true if the cell is not solved.
    pub const fn is_unsolved(&self, cell: Cell) -> bool {
        !self.solved_cells.has(cell)
    }

    /// Returns the number of unsolved cells in the puzzle.
    pub const fn unsolved_count(&self) -> usize {
        81 - self.solved_cells.len()
    }

    /// Returns the set of all unsolved cells.
    pub fn unsolved(&self) -> CellSet {
        !self.solved_cells
    }

    /// Returns an iterator of all unsolved cells with their candidates.
    pub fn unsolved_iter(&self) -> impl Iterator<Item = (Cell, DigitSet)> + '_ {
        self.unsolved()
            .into_iter()
            .map(|cell| (cell, self.candidates(cell)))
    }

    /// Returns true if every cell on the board is solved (given or placed).
    pub const fn is_fully_solved(&self) -> bool {
        self.solved_cells.is_full()
    }

    /// Returns true if the cell has a digit.
    pub const fn is_solved(&self, cell: Cell) -> bool {
        self.solved_cells.has(cell)
    }

    /// Returns the number of solved cells in the puzzle, including givens.
    pub const fn solved_count(&self) -> usize {
        self.solved_cells.len()
    }

    /// Returns the set of all solved cells, including givens.
    pub const fn solved(&self) -> CellSet {
        self.solved_cells
    }

    /// Returns the set of all cells solved with the digit, including givens.
    pub const fn solved_with(&self, digit: Digit) -> CellSet {
        self.solved_cells_by_digit[digit.usize()]
    }

    /// Returns an iterator of all solved cells with their digit, including givens.
    pub fn solved_iter(&self) -> impl Iterator<Item = (Cell, Digit)> + '_ {
        self.solved_cells
            .into_iter()
            .map(|cell| (cell, self.value(cell).digit().unwrap()))
    }

    /// Returns an iterator of a subset of solved cells with their digit, including givens.
    pub fn solved_subset_iter(&self, cells: CellSet) -> impl Iterator<Item = (Cell, Digit)> + '_ {
        (cells & self.solved_cells)
            .into_iter()
            .map(|cell| (cell, self.value(cell).digit().unwrap()))
    }

    /// Returns the set of digits to which any of the cells is set.
    pub fn solved_subset_digits(&self, cells: CellSet) -> DigitSet {
        cells.iter().fold(DigitSet::empty(), |acc, cell| {
            self.value(cell).digit().map_or(acc, |d| acc + d)
        })
    }

    /// Returns true if a cell in the house has the digit.
    pub fn is_digit_in_house(&self, house: House, digit: Digit) -> bool {
        !(self.solved_cells_by_digit[digit.usize()] & house.cells()).is_empty()
    }

    /// Returns true if the cell is a given.
    pub const fn is_given(&self, cell: Cell) -> bool {
        self.givens.has(cell)
    }

    /// Returns the number of givens in the puzzle.
    pub const fn given_count(&self) -> usize {
        self.givens.len()
    }

    /// Returns the set of all givens.
    pub const fn givens(&self) -> CellSet {
        self.givens
    }

    /// Returns the set of all cells given the digit.
    pub fn givens_with(&self, digit: Digit) -> CellSet {
        self.givens & self.solved_cells_by_digit[digit.usize()]
    }

    /// Returns true if the cell could not have been solved by the digit due to a peer with the same given.
    pub fn is_blocked_by_given(&self, cell: Cell, digit: Digit) -> bool {
        !(cell.peers() & self.givens & self.solved_cells_by_digit[digit.usize()]).is_empty()
    }

    /// Returns true if the cell is placed.
    pub const fn is_placed(&self, cell: Cell) -> bool {
        self.solved_cells.has(cell) && !self.givens.has(cell)
    }

    /// Returns the number of placed cells in the puzzle.
    pub const fn placed_count(&self) -> usize {
        self.solved_cells.len() - self.givens.len()
    }

    /// Returns the set of all placed cells.
    pub fn placed(&self) -> CellSet {
        self.solved_cells - self.givens
    }

    /// Returns the set of all placed cells for the digit.
    pub fn placed_with(&self, digit: Digit) -> CellSet {
        self.solved_cells_by_digit[digit.usize()] - self.givens
    }

    /// Returns true if every cell in the house has a digit.
    pub fn is_house_solved(&self, house: House) -> bool {
        (!self.solved_cells & house.cells()).is_empty()
    }

    /// Returns the value of the cell, either a digit or none.
    pub const fn value(&self, cell: Cell) -> Value {
        self.values[cell.usize()]
    }

    /// Sets the cell to the digit, marks it as a given,
    /// and returns the change along with any follow-up actions found.
    ///
    /// See [`Board::set_digit()`] for more details.
    pub fn set_given(&mut self, cell: Cell, digit: Digit, effects: &mut Effects) -> Change {
        let change = self.set_digit(cell, digit, effects);
        if change.changed() {
            self.givens += cell;
        }
        change
    }

    /// Sets the cell to the digit, marks it as placed,
    /// and returns the change along with any follow-up actions found.
    ///
    /// See [`Board::set_digit()`] for more details.
    pub fn set_placed(&mut self, cell: Cell, digit: Digit, effects: &mut Effects) -> Change {
        self.set_digit(cell, digit, effects)
    }

    /// Sets the cell to the digit and returns the change
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
    /// Returns no change with no actions or errors
    /// if the digit is not a candidate for the cell.
    fn set_digit(&mut self, cell: Cell, digit: Digit, effects: &mut Effects) -> Change {
        if let Some(current) = self.value(cell).digit() {
            if current == digit {
                return Change::None;
            } else {
                effects.add_error(Error::AlreadySolved(cell, digit, current));
                return Change::Invalid;
            }
        } else if !self.is_candidate(cell, digit) {
            effects.add_error(Error::NotCandidate(cell, digit));
            return Change::Invalid;
        }

        if let Some(rectangles) = creates_deadly_rectangles(self, cell, digit) {
            rectangles.into_iter().for_each(|r| {
                effects.add_error(Error::DeadlyRectangle(r));
            });
            // Do not return Invalid since the move itself is valid
        }

        self.values[cell.usize()] = digit.value();
        self.solved_cells += cell;
        self.solved_cells_by_digit[digit.usize()] += cell;
        self.candidate_cells_by_digit[digit.usize()] -= cell;

        let mut change = Change::Valid;
        let mut candidates = self.candidate_digits_by_cell[cell.usize()];
        self.candidate_digits_by_cell[cell.usize()] = DigitSet::empty();
        self.cells_with_n_candidates[candidates.len()] -= cell;
        self.cells_with_n_candidates[0] += cell;
        candidates -= digit;
        for candidate in candidates {
            self.candidate_cells_by_digit[candidate.usize()] -= cell;
            change &= self.remove_candidate_cell_from_houses(cell, candidate, effects);
        }

        for peer in self.candidate_cells_by_digit[digit.usize()] & cell.peers() {
            change &= self.remove_candidate(peer, digit, effects);
            // effects.add_erase(Strategy::Peer, peer, digit)
        }

        change
    }

    /// Returns a new pseudo cell with the given cells and their candidates.
    pub fn pseudo_cell(&self, cells: CellSet) -> PseudoCell {
        PseudoCell::new(cells, self.combined_candidates(cells))
    }

    /// Returns true if the cell has the candidate.
    pub const fn is_candidate(&self, cell: Cell, digit: Digit) -> bool {
        self.candidate_digits_by_cell[cell.usize()].has(digit)
    }

    /// Returns the set of candidates for the cell.
    pub const fn candidates(&self, cell: Cell) -> DigitSet {
        self.candidate_digits_by_cell[cell.usize()]
    }

    /// Returns the set of combined candidates for the cells.
    pub fn combined_candidates(&self, cells: CellSet) -> DigitSet {
        cells
            .iter()
            .fold(DigitSet::empty(), |acc, cell| acc | self.candidates(cell))
    }

    /// Returns the set of common candidates for the cells.
    pub fn common_candidates(&self, cells: CellSet) -> DigitSet {
        if cells.is_empty() {
            return DigitSet::empty();
        }
        cells
            .iter()
            .fold(DigitSet::full(), |acc, cell| acc & self.candidates(cell))
    }

    /// Returns all cells that have N candidates.
    pub const fn cells_with_n_candidates(&self, n: usize) -> CellSet {
        // TODO All calls are n <= 3, so maybe only track up to 3 candidates?
        debug_assert!(n <= 9);
        self.cells_with_n_candidates[n]
    }

    /// Returns an iterator of unsolved cells with N candidates with their candidates.
    pub fn cells_with_n_candidates_iter(
        &self,
        n: usize,
    ) -> impl Iterator<Item = (Cell, DigitSet)> + '_ {
        self.cells_with_n_candidates(n)
            .iter()
            .map(|cell| (cell, self.candidates(cell)))
    }

    /// Returns the set of cells that have the candidate.
    pub const fn candidate_cells(&self, digit: Digit) -> CellSet {
        self.candidate_cells_by_digit[digit.usize()]
    }

    /// Returns the set of cells in the house that have the candidate.
    pub fn house_candidate_cells(&self, house: House, digit: Digit) -> CellSet {
        house.cells() & self.candidate_cells(digit)
    }

    /// Returns the strong links for the digit as peer pairs.
    pub fn strong_links_for_digit(&self, digit: Digit) -> PeerSet {
        let mut links = PeerSet::empty();

        for house in House::iter() {
            if let Some((a, b)) = self.house_candidate_cells(house, digit).as_pair() {
                links += (a, b);
            }
        }

        links
    }

    /// Returns the strong links for all digits.
    pub fn strong_links(&self) -> [PeerSet; 9] {
        let mut links = [PeerSet::empty(); 9];

        for digit in Digit::iter() {
            links[digit.usize()] = self.strong_links_for_digit(digit);
        }

        links
    }

    /// Removes the candidate from the cell and returns change
    /// along with any follow-up actions found.
    ///
    /// The cell is removed as a candidate from its three houses.
    ///
    /// If any errors are caused while removing the candidate,
    /// they are returned with the actions, and the puzzle
    /// will be left in an unsolvable state, but the internal
    /// state will be consistent.
    ///
    /// Returns no change with no actions or errors
    /// if the digit is not a candidate for the cell.
    pub fn remove_candidate(
        &mut self,
        cell: Cell,
        candidate: Digit,
        effects: &mut Effects,
    ) -> Change {
        let candidates = &mut self.candidate_digits_by_cell[cell.usize()];
        if !candidates[candidate] {
            return Change::None;
        }

        let size = candidates.len();
        *candidates -= candidate;
        self.cells_with_n_candidates[size] -= cell;
        self.cells_with_n_candidates[size - 1] += cell;
        self.candidate_cells_by_digit[candidate.usize()] -= cell;

        let mut change = Change::Valid;
        if candidates.is_empty() {
            effects.add_error(Error::UnsolvableCell(cell));
            change = Change::Invalid;
        } else if let Some(single) = candidates.as_single() {
            effects.add_set(Strategy::NakedSingle, cell, single);
        }

        change & self.remove_candidate_cell_from_houses(cell, candidate, effects)
    }

    /// Removes the cell as a candidate for the digit
    /// from its three houses and returns the change
    /// along with any follow-up actions found.
    ///
    /// If any errors are caused while removing the candidate,
    /// they are returned with the actions, and the puzzle
    /// will be left in an unsolvable state, but the internal
    /// state will be consistent.
    fn remove_candidate_cell_from_houses(
        &mut self,
        cell: Cell,
        digit: Digit,
        effects: &mut Effects,
    ) -> Change {
        let mut change = Change::None;

        for house in cell.houses() {
            if self.is_digit_in_house(house, digit) {
                continue;
            }

            change &= Change::Valid;
            let candidates = self.house_candidate_cells(house, digit);
            if candidates.is_empty() {
                effects.add_error(Error::UnsolvableHouse(house, digit));
                change &= Change::Invalid;
            } else if let Some(single) = candidates.as_single() {
                effects.add_set(Strategy::HiddenSingle, single, digit);
            }
        }

        change
    }

    /// Removes the candidates from the cell and returns the change
    /// along with any follow-up actions found.
    ///
    /// See [`Board::remove_candidate()`] for more details.
    pub fn remove_candidates(
        &mut self,
        cell: Cell,
        digits: DigitSet,
        effects: &mut Effects,
    ) -> Change {
        digits.iter().fold(Change::None, |change, digit| {
            change & self.remove_candidate(cell, digit, effects)
        })
    }

    /// Removes the candidate from the cells and returns the change
    /// along with any follow-up actions found.
    ///
    /// See [`Board::remove_candidate()`] for more details.
    pub fn remove_candidate_from_cells(
        &mut self,
        cells: CellSet,
        digit: Digit,
        effects: &mut Effects,
    ) -> Change {
        cells.iter().fold(Change::None, |change, cell| {
            change & self.remove_candidate(cell, digit, effects)
        })
    }

    /// Removes the candidates from the cells and returns the change
    /// along with any follow-up actions found.
    ///
    /// See [`Board::remove_candidate()`] for more details.
    pub fn remove_candidates_from_cells(
        &mut self,
        cells: CellSet,
        digits: DigitSet,
        effects: &mut Effects,
    ) -> Change {
        cells.iter().fold(Change::None, |change, cell| {
            change
                & digits.iter().fold(Change::None, |change, digit| {
                    change & self.remove_candidate(cell, digit, effects)
                })
        })
    }

    /// Returns a new board with the digits of this board
    /// set as givens for the specified cells.
    ///
    /// If any specified cell is not solved in this board,
    /// it is left unsolved in the returned board.
    pub fn with_givens(&self, pattern: CellSet) -> (Board, Effects) {
        (pattern & self.solved()).iter().fold(
            (Board::new(), Effects::new()),
            |(mut b, mut e), c| {
                b.set_given(c, self.value(c).digit().unwrap(), &mut e);
                (b, e)
            },
        )
    }

    /// Returns a new board with the digits of this board
    /// except for the one in the given cell.
    pub fn without(&self, cell: Cell) -> (Board, Effects) {
        self.solved_iter().filter(|(c, _)| *c != cell).fold(
            (Board::new(), Effects::new()),
            |(mut b, mut e), (c, d)| {
                b.set_given(c, d, &mut e);
                (b, e)
            },
        )
    }

    /// Returns the packed string format of the digits of this board
    /// with a period for each unsolved cell and no spacing between rows.
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

#[cfg(test)]
mod test {
    use itertools::Itertools;

    use crate::io::{Parse, Parser};
    use crate::layout::Shape;
    use crate::testing::strip_leading_whitespace;
    use crate::*;

    use super::*;

    fn with_given_and_placed() -> Board {
        let mut board = Board::new();
        let mut effects = Effects::new();
        board.set_given(cell!(A1), digit!(1), &mut effects);
        board.set_placed(cell!(B2), digit!(2), &mut effects);
        board
    }

    fn fixture() -> Board {
        Parse::grid().parse_simple(
            strip_leading_whitespace(
                "
                +-----------------+--------------------+-----------------+
                | 48   59   2     | 1459   18    1589  | 3    7    6     |
                | 478  1    468   | 24679  3     2689  | 5    248  248   |
                | 3478 3567 4568  | 124567 12678 12568 | 148  9    248   |
                +-----------------+--------------------+-----------------+
                | 9    367  46    | 8      5     26    | 467  24   1     |
                | 78   567  1568  | 3      126   4     | 6789 258  25789 |
                | 2    56   14568 | 16     9     7     | 468  458  3     |
                +-----------------+--------------------+-----------------+
                | 6    8    9     | 1257   127   1235  | 147  1345 457   |
                | 5    2    3     | 179    4     189   | 1789 6    789   |
                | 1    4    7     | 569    68    35689 | 2    358  589   |
                +-----------------+--------------------+-----------------+
            ",
            )
            .as_str(),
        )
    }

    #[test]
    fn test_new() {
        let f = Board::new();

        assert_eq!(f.unsolved_count(), 81);
        assert_eq!(f.unsolved(), all_cells![]);
        assert_eq!(f.solved_count(), 0);
        assert_eq!(f.solved(), cells![]);
        assert_eq!(f.solved_subset_digits(all_cells![]), digits![]);

        assert_eq!(f.given_count(), 0);
        assert_eq!(f.givens(), cells![]);

        assert_eq!(f.is_fully_solved(), false);
        assert_eq!(f.placed_count(), 0);
        assert_eq!(f.placed(), cells![]);

        for cell in Cell::iter() {
            assert_eq!(f.is_unsolved(cell), true);
            assert_eq!(f.is_solved(cell), false);
            assert_eq!(f.is_given(cell), false);
            assert_eq!(f.is_placed(cell), false);
            assert_eq!(f.value(cell), Value::none());
            assert_eq!(f.candidates(cell), DigitSet::full());
        }

        for digit in Digit::iter() {
            assert_eq!(f.candidate_cells(digit), all_cells![]);
        }

        for house in House::iter() {
            assert_eq!(f.is_house_solved(house), false);
            for digit in Digit::iter() {
                assert_eq!(f.is_digit_in_house(house, digit), false);
            }
        }
    }

    #[test]
    fn test_change_and_invalid_paths() {
        assert_eq!(Change::Invalid, Change::Valid.and(Change::Invalid));
        assert_eq!(Change::Invalid, Change::Invalid.and(Change::Valid));
        assert_eq!(Change::Valid, Change::Valid.and(Change::Valid));
        assert_eq!(Change::None, Change::None.and(Change::None));
    }

    #[test]
    fn test_unsolved_and_solved_iters() {
        let f = with_given_and_placed();

        let unsolved = f.unsolved_iter().map(|(cell, _)| cell).collect_vec();
        assert_eq!(79, unsolved.len());
        assert!(!unsolved.contains(&cell!(A1)));
        assert!(!unsolved.contains(&cell!(B2)));

        let solved = f.solved_iter().collect_vec();
        assert_eq!(vec![(cell!(A1), digit!(1)), (cell!(B2), digit!(2))], solved);
    }

    #[test]
    fn test_solved_givens_and_placed_sets() {
        let f = with_given_and_placed();

        assert_eq!(cells![A1], f.solved_with(digit!(1)));
        assert_eq!(cells![B2], f.solved_with(digit!(2)));

        assert_eq!(cells![A1], f.givens_with(digit!(1)));
        assert_eq!(cells![], f.givens_with(digit!(2)));

        assert_eq!(cells![B2], f.placed_with(digit!(2)));
        assert_eq!(cells![], f.placed_with(digit!(1)));
    }

    #[test]
    fn test_blocked_by_given() {
        let mut board = Board::new();
        let mut effects = Effects::new();
        board.set_given(cell!(A1), digit!(1), &mut effects);

        assert!(board.is_blocked_by_given(cell!(A2), digit!(1)));
        assert!(board.is_blocked_by_given(cell!(B2), digit!(1)));
        assert!(!board.is_blocked_by_given(cell!(H9), digit!(1)));
    }

    #[test]
    fn test_set_digit_error_paths() {
        let mut board = Board::new();
        let mut effects = Effects::new();
        board.set_given(cell!(A1), digit!(1), &mut effects);

        let change = board.set_placed(cell!(A1), digit!(2), &mut effects);
        assert_eq!(Change::Invalid, change);
        assert!(effects
            .errors()
            .contains(&Error::AlreadySolved(cell!(A1), digit!(2), digit!(1))));

        let mut board = Board::new();
        let mut effects = Effects::new();
        board.remove_candidate(cell!(A2), digit!(3), &mut effects);

        let change = board.set_placed(cell!(A2), digit!(3), &mut effects);
        assert_eq!(Change::Invalid, change);
        assert!(effects
            .errors()
            .contains(&Error::NotCandidate(cell!(A2), digit!(3))));
    }

    #[test]
    fn test_remove_candidate_unsolvable_cell() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        for digit in [1u8, 2, 3, 4, 5, 6, 7, 8] {
            board.remove_candidate(cell!(A1), Digit::from_ordinal(digit), &mut effects);
        }

        let change = board.remove_candidate(cell!(A1), digit!(9), &mut effects);
        assert_eq!(Change::Invalid, change);
        assert!(effects.errors().contains(&Error::UnsolvableCell(cell!(A1))));
        assert!(board.candidates(cell!(A1)).is_empty());
    }

    #[test]
    fn test_remove_candidate_unsolvable_house() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let row_cells = row!(A).cells();
        for cell in row_cells {
            board.remove_candidate(cell, digit!(1), &mut effects);
        }

        assert!(effects
            .errors()
            .contains(&Error::UnsolvableHouse(row!(A), digit!(1))));
    }

    #[test]
    fn test_remove_candidates_helpers() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let change = board.remove_candidates(cell!(A1), digits![1 2], &mut effects);
        assert_eq!(Change::Valid, change);
        assert!(!board.is_candidate(cell!(A1), digit!(1)));
        assert!(!board.is_candidate(cell!(A1), digit!(2)));

        let change = board.remove_candidate_from_cells(cells![A1 A2], digit!(3), &mut effects);
        assert_eq!(Change::Valid, change);
        assert!(!board.is_candidate(cell!(A1), digit!(3)));
        assert!(!board.is_candidate(cell!(A2), digit!(3)));
    }

    #[test]
    fn test_with_givens_and_without() {
        let board = with_given_and_placed();
        let (with_givens, effects) = board.with_givens(cells![A1 A2]);

        assert!(!effects.has_errors());
        assert!(with_givens.is_given(cell!(A1)));
        assert_eq!(Some(digit!(1)), with_givens.value(cell!(A1)).digit());
        assert!(with_givens.value(cell!(A2)).is_none());

        let (without, _) = board.without(cell!(A1));
        assert!(without.is_given(cell!(B2)));
        assert!(without.value(cell!(A1)).is_none());
    }

    #[test]
    fn test_packed_string_and_display() {
        let board = with_given_and_placed();

        let packed = board.packed_string();
        assert_eq!(81, packed.len());
        assert_eq!(Some('1'), packed.chars().nth(0));
        assert_eq!(Some('.'), packed.chars().nth(1));
        assert_eq!(Some('2'), packed.chars().nth(10));

        let display = format!("{}", board);
        assert!(!display.is_empty());
    }

    #[test]
    fn test_parsed() {
        let f = fixture();
        let solved = cells![
            A3 A7 A8 A9
            B2 B5 B7
            C8 D1 D4 D5 D9
            E4 E6
            F1 F5 F6 F9
            G1 G2 G3
            H1 H2 H3 H5 H8
            J1 J2 J3 J7
        ];

        assert_eq!(f.unsolved_count(), 81 - solved.len());
        assert_eq!(f.unsolved(), all_cells![] - solved);
        assert_eq!(f.solved_count(), solved.len());
        assert_eq!(f.solved(), solved);
        assert_eq!(f.solved_subset_digits(all_cells![]), DigitSet::full());

        assert_eq!(f.given_count(), 0);
        assert_eq!(f.givens(), cells![]);

        assert_eq!(f.is_fully_solved(), false);
        assert_eq!(f.placed_count(), solved.len());
        assert_eq!(f.placed(), solved);

        for cell in solved {
            assert_eq!(f.is_unsolved(cell), false);
            assert_eq!(f.is_solved(cell), true);
            assert_eq!(f.is_given(cell), false);
            assert_eq!(f.is_placed(cell), true);
            assert_eq!(f.value(cell).is_digit(), true);
            assert_eq!(f.candidates(cell), digits![]);
        }
    }

    #[test]
    fn test_is_candidate() {
        let f = fixture();

        assert_eq!(f.is_candidate(cell!(A1), digit!(4)), true);
        assert_eq!(f.is_candidate(cell!(A1), digit!(8)), true);
        assert_eq!(f.is_candidate(cell!(C3), digit!(4)), true);
        assert_eq!(f.is_candidate(cell!(C3), digit!(5)), true);
        assert_eq!(f.is_candidate(cell!(C3), digit!(6)), true);
        assert_eq!(f.is_candidate(cell!(C3), digit!(8)), true);
        assert_eq!(f.is_candidate(cell!(A1), digit!(1)), false);
        assert_eq!(f.is_candidate(cell!(A1), digit!(2)), false);
        assert_eq!(f.is_candidate(cell!(A1), digit!(3)), false);
        assert_eq!(f.is_candidate(cell!(A1), digit!(5)), false);
        assert_eq!(f.is_candidate(cell!(A1), digit!(6)), false);
        assert_eq!(f.is_candidate(cell!(A1), digit!(7)), false);
        assert_eq!(f.is_candidate(cell!(A1), digit!(9)), false);

        assert_eq!(f.is_candidate(cell!(H1), digit!(5)), false);
    }

    #[test]
    fn test_candidates() {
        let f = fixture();

        assert_eq!(f.candidates(cell!(A1)), digits![4 8]);
        assert_eq!(f.candidates(cell!(C3)), digits![4 5 6 8]);
        assert_eq!(f.candidates(cell!(D1)), digits![]);
    }

    #[test]
    fn test_combined_candidates() {
        let f = fixture();

        assert_eq!(f.combined_candidates(cells![]), digits![]);
        assert_eq!(f.combined_candidates(all_cells![]), DigitSet::full());
        assert_eq!(f.combined_candidates(cells![A1 A2]), digits![4 5 8 9]);
        assert_eq!(
            f.combined_candidates(cells![A1 A2 A3 A4]),
            digits![1 4 5 8 9]
        );
    }

    #[test]
    fn test_common_candidates() {
        let f = fixture();

        assert_eq!(f.common_candidates(cells![]), digits![]);
        assert_eq!(f.common_candidates(all_cells![]), digits![]);
        assert_eq!(f.common_candidates(cells![A2 A4]), digits![5 9]);
        assert_eq!(f.common_candidates(cells![A1 A2 A3 A4]), digits![]);
    }

    #[test]
    fn test_cells_with_n_candidates() {
        let f = fixture();

        assert_eq!(f.cells_with_n_candidates(0), f.solved());
        assert_eq!(
            f.cells_with_n_candidates(0),
            cells![
                A3 A7 A8 A9
                B2 B5 B7
                C8
                D1 D4 D5 D9
                E4 E6
                F1 F5 F6 F9
                G1 G2 G3 H1 H2 H3 H5 H8
                J1 J2 J3 J7
            ]
        );
        assert_eq!(f.cells_with_n_candidates(1), cells![]);
        assert_eq!(
            f.cells_with_n_candidates(2),
            cells![
                A1 A2 A5
                D3 D6 D8
                E1
                F2 F4
                J5
            ]
        );
        assert_eq!(
            f.cells_with_n_candidates(3),
            cells![
                B1 B3 B8 B9
                C7 C9
                D2 D7
                E2 E5 E8
                F7 F8
                G5 G7 G9
                H4 H6 H9
                J4 J8 J9
            ]
        );
        assert_eq!(
            f.cells_with_n_candidates(4),
            cells![
                A4 A6
                B6
                C1 C2 C3
                E3 E7
                G4 G6 G8
                H7
            ]
        );
        assert_eq!(
            f.cells_with_n_candidates(5),
            cells![
                B4
                C5 C6
                E9
                F3
                J6
            ]
        );
        assert_eq!(f.cells_with_n_candidates(6), cells![C4]);
        assert_eq!(f.cells_with_n_candidates(7), cells![]);
        assert_eq!(f.cells_with_n_candidates(8), cells![]);
        assert_eq!(f.cells_with_n_candidates(9), cells![]);
    }

    #[test]
    fn test_cells_with_n_candidates_iter() {
        let f = fixture();

        assert_eq!(
            f.cells_with_n_candidates_iter(5).collect_vec(),
            vec![
                (cell!(B4), digits![2 4 6 7 9]),
                (cell!(C5), digits![1 2 6 7 8]),
                (cell!(C6), digits![1 2 5 6 8]),
                (cell!(E9), digits![2 5 7 8 9]),
                (cell!(F3), digits![1 4 5 6 8]),
                (cell!(J6), digits![3 5 6 8 9]),
            ]
        );
        assert_eq!(
            f.cells_with_n_candidates_iter(6).collect_vec(),
            vec![(cell!(C4), digits![1 2 4 5 6 7])]
        );
        assert_eq!(
            f.cells_with_n_candidates_iter(7).collect_vec().is_empty(),
            true
        );
    }

    #[test]
    fn test_candidate_cells() {
        let f = fixture();

        assert_eq!(
            f.candidate_cells(digit!(1)),
            cells![
                A4 A5 A6
                C4 C5 C6 C7
                E3 E5
                F3 F4
                G4 G5 G6 G7 G8
                H4 H6 H7
            ]
        );
    }

    #[test]
    fn test_house_candidate_cells() {
        let f = fixture();

        assert_eq!(
            f.house_candidate_cells(row!(C), digit!(1)),
            cells![C4 C5 C6 C7]
        );
    }
}
