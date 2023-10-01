use super::*;
use itertools::Itertools;

pub fn find_hidden_pairs(board: &Board) -> Option<Effects> {
    find_hidden_tuples(board, 2, Strategy::NakedPair)
}

pub fn find_hidden_triples(board: &Board) -> Option<Effects> {
    find_hidden_tuples(board, 3, Strategy::NakedTriple)
}

pub fn find_hidden_quads(board: &Board) -> Option<Effects> {
    find_hidden_tuples(board, 4, Strategy::NakedQuad)
}

pub fn find_hidden_tuples(board: &Board, size: usize, strategy: Strategy) -> Option<Effects> {
    let mut effects = Effects::new();

    House::iter().for_each(|house| {
        Known::iter()
            .map(|k| (k, house.cells() & board.candidate_cells(k)))
            .filter(|(_, candidates)| 2 <= candidates.size() && candidates.size() <= size)
            .combinations(size)
            .for_each(|candidates| {
                let cell_sets = candidates.iter().map(|(_, cs)| *cs).collect::<Vec<_>>();
                let cells = cell_sets.iter().copied().union();
                if cells.size() != size
                    || is_degenerate(&cell_sets, size, 2)
                    || is_degenerate(&cell_sets, size, 3)
                {
                    return;
                }

                let knowns = candidates.iter().map(|(k, _)| *k).union();
                let mut action = Action::new(strategy);

                cells
                    .iter()
                    .for_each(|c| action.erase_knowns(c, board.candidates(c) - knowns));

                if !action.is_empty() {
                    // TODO check for dupes (same pair in block and row or column)
                    effects.add_action(action);
                }
            });
    });

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
            .map(|sets| sets.into_iter().copied().union())
            .any(|set| (set.size()) <= smaller_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::cells::cell::cell;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::values::known_set::knowns;
    use crate::layout::Cell;

    #[test]
    fn hidden_pairs() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let cells = cells!("A1 A2 A4 A5 A6 A8 A9");
        let knowns = knowns!("1 2");
        board.remove_candidates_from_cells(cells, knowns, &mut effects);

        find_hidden_pairs(&board).unwrap().apply_all(&mut board);

        assert_eq!(knowns, board.candidates(cell!("A3")));
        assert_eq!(knowns, board.candidates(cell!("A7")));
        assert_eq!(-knowns, board.candidates(cell!("A2")));
        assert_eq!(-knowns, board.candidates(cell!("A6")));
        assert_eq!(-knowns, board.candidates(cell!("A9")));
    }

    #[test]
    fn hidden_triples() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let cells = cells!("A1 A2 A4 A6 A8 A9");
        let knowns = knowns!("1 2 3");
        board.remove_candidates_from_cells(cells, knowns, &mut effects);

        find_hidden_triples(&board).unwrap().apply_all(&mut board);

        assert_eq!(knowns, board.candidates(cell!("A3")));
        assert_eq!(knowns, board.candidates(cell!("A5")));
        assert_eq!(knowns, board.candidates(cell!("A7")));
        assert_eq!(-knowns, board.candidates(cell!("A2")));
        assert_eq!(-knowns, board.candidates(cell!("A6")));
        assert_eq!(-knowns, board.candidates(cell!("A9")));
    }

    #[test]
    fn hidden_quads() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let cells = cells!("A2 A4 A6 A8 A9");
        let knowns = knowns!("1 2 3 4");
        board.remove_candidates_from_cells(cells, knowns, &mut effects);

        find_hidden_quads(&board).unwrap().apply_all(&mut board);

        assert_eq!(knowns, board.candidates(cell!("A1")));
        assert_eq!(knowns, board.candidates(cell!("A3")));
        assert_eq!(knowns, board.candidates(cell!("A5")));
        assert_eq!(knowns, board.candidates(cell!("A7")));
        assert_eq!(-knowns, board.candidates(cell!("A2")));
        assert_eq!(-knowns, board.candidates(cell!("A6")));
        assert_eq!(-knowns, board.candidates(cell!("A9")));
    }
}
