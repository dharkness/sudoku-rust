use itertools::Itertools;

use super::*;

pub fn find_hidden_pairs(board: &Board, single: bool) -> Option<Effects> {
    find_hidden_tuples(board, single, 2, Strategy::HiddenPair)
}

pub fn find_hidden_triples(board: &Board, single: bool) -> Option<Effects> {
    find_hidden_tuples(board, single, 3, Strategy::HiddenTriple)
}

pub fn find_hidden_quads(board: &Board, single: bool) -> Option<Effects> {
    find_hidden_tuples(board, single, 4, Strategy::HiddenQuad)
}

pub fn find_hidden_tuples(
    board: &Board,
    single: bool,
    size: usize,
    strategy: Strategy,
) -> Option<Effects> {
    let mut effects = Effects::new();

    for house in House::iter() {
        let house_cells = house.cells();
        for candidates in Digit::iter()
            .map(|d| (d, house_cells & board.candidate_cells(d)))
            .filter(|(_, cells)| (2..=size).contains(&cells.len()))
            .combinations(size)
        {
            let cell_sets = candidates.iter().map(|(_, cs)| *cs).collect_vec();
            let tuple_cells = cell_sets.iter().copied().union_cells();
            if tuple_cells.len() != size
                || is_degenerate(&cell_sets, size, 2)
                || is_degenerate(&cell_sets, size, 3)
            {
                continue;
            }

            let tuple_digits = candidates.iter().map(|(d, _)| *d).union_digits();
            let mut action = Action::new(strategy);

            tuple_cells
                .iter()
                .for_each(|c| action.erase_digits(c, board.candidates(c) - tuple_digits));
            tuple_digits.iter().for_each(|d| {
                action.clue_cells_for_digit(
                    Verdict::Secondary,
                    board.house_candidate_cells(house, d),
                    d,
                );
            });
            (house_cells - tuple_cells).iter().for_each(|c| {
                action.clue_cell_for_digits(Verdict::Related, c, tuple_digits);
            });

            // TODO check for dupes (same pair in block and row or column)
            if effects.add_action(action) && single {
                return Some(effects);
            }
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

pub fn is_degenerate(cell_sets: &[CellSet], size: usize, smaller_size: usize) -> bool {
    size > smaller_size
        && cell_sets
            .iter()
            .combinations(smaller_size)
            .map(|sets| sets.into_iter().copied().union_cells())
            .any(|set| (set.len()) <= smaller_size)
}

#[cfg(test)]
mod tests {
    use crate::layout::Cell;
    use crate::*;

    use super::*;

    #[test]
    fn hidden_pairs() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let cells = cells![A1 A2 A4 A5 A6 A8 A9];
        let digits = digits![1 2];
        board.remove_candidates_from_cells(cells, digits, &mut effects);

        find_hidden_pairs(&board, false)
            .unwrap()
            .apply_all(&mut board);

        assert_eq!(digits, board.candidates(cell!(A3)));
        assert_eq!(digits, board.candidates(cell!(A7)));
        assert_eq!(!digits, board.candidates(cell!(A2)));
        assert_eq!(!digits, board.candidates(cell!(A6)));
        assert_eq!(!digits, board.candidates(cell!(A9)));
    }

    #[test]
    fn hidden_triples() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let cells = cells![A1 A2 A4 A6 A8 A9];
        let digits = digits![1 2 3];
        board.remove_candidates_from_cells(cells, digits, &mut effects);

        find_hidden_triples(&board, false)
            .unwrap()
            .apply_all(&mut board);

        assert_eq!(digits, board.candidates(cell!(A3)));
        assert_eq!(digits, board.candidates(cell!(A5)));
        assert_eq!(digits, board.candidates(cell!(A7)));
        assert_eq!(!digits, board.candidates(cell!(A2)));
        assert_eq!(!digits, board.candidates(cell!(A6)));
        assert_eq!(!digits, board.candidates(cell!(A9)));
    }

    #[test]
    fn hidden_quads() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let cells = cells![A2 A4 A6 A8 A9];
        let digits = digits![1 2 3 4];
        board.remove_candidates_from_cells(cells, digits, &mut effects);

        find_hidden_quads(&board, false)
            .unwrap()
            .apply_all(&mut board);

        assert_eq!(digits, board.candidates(cell!(A1)));
        assert_eq!(digits, board.candidates(cell!(A3)));
        assert_eq!(digits, board.candidates(cell!(A5)));
        assert_eq!(digits, board.candidates(cell!(A7)));
        assert_eq!(!digits, board.candidates(cell!(A2)));
        assert_eq!(!digits, board.candidates(cell!(A6)));
        assert_eq!(!digits, board.candidates(cell!(A9)));
    }
}
