use super::*;

pub fn find_naked_pairs(board: &Board) -> Option<Effects> {
    find_naked_tuples(board, 2, Strategy::NakedPair)
}

pub fn find_naked_triples(board: &Board) -> Option<Effects> {
    find_naked_tuples(board, 3, Strategy::NakedTriple)
}

pub fn find_naked_quads(board: &Board) -> Option<Effects> {
    find_naked_tuples(board, 4, Strategy::NakedQuad)
}

fn find_naked_tuples(board: &Board, size: usize, strategy: Strategy) -> Option<Effects> {
    let mut effects = Effects::new();

    for house in House::iter() {
        let house_cells = house.cells();
        house_cells
            .iter()
            .map(|cell| (cell, board.candidates(cell)))
            .filter(|(_, candidates)| 2 <= candidates.len() && candidates.len() <= size)
            .combinations(size)
            .for_each(|candidates| {
                let known_sets = candidates.iter().map(|(_, ks)| *ks).collect::<Vec<_>>();
                let knowns = known_sets.iter().copied().union() as KnownSet;
                if knowns.len() != size
                    || is_degenerate(&known_sets, size, 2)
                    || is_degenerate(&known_sets, size, 3)
                {
                    return;
                }

                let cells = house_cells - candidates.iter().map(|(c, _)| *c).union();
                let mut action = Action::new(strategy);

                knowns
                    .iter()
                    .for_each(|k| action.erase_cells(cells & board.candidate_cells(k), k));

                if !action.is_empty() {
                    // TODO check for dupes (same pair in block and row or column)
                    effects.add_action(action);
                }
            });
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

pub fn is_degenerate(known_sets: &[KnownSet], size: usize, smaller_size: usize) -> bool {
    size > smaller_size
        && known_sets
            .iter()
            .combinations(smaller_size)
            .map(|sets| sets.into_iter().copied().union())
            .any(|set| (set.len()) <= smaller_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::cells::cell::cell;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::values::known_set::knowns;

    #[test]
    fn naked_pairs() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let knowns = knowns!("1 2 3 4 5 6 7");
        board.remove_candidates_from_cells(cells!("A1 A2"), knowns, &mut effects);

        find_naked_pairs(&board).unwrap().apply_all(&mut board);

        assert_eq!(-knowns, board.candidates(cell!("A1")));
        assert_eq!(-knowns, board.candidates(cell!("A2")));
        assert_eq!(knowns, board.candidates(cell!("A5")));
        assert_eq!(knowns, board.candidates(cell!("B3")));
        assert_eq!(knowns, board.candidates(cell!("C2")));
    }

    #[test]
    fn naked_triples() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let knowns = knowns!("1 2 3 4 5 6");
        board.remove_candidates_from_cells(cells!("A1 A2 A5"), knowns, &mut effects);

        find_naked_triples(&board).unwrap().apply_all(&mut board);

        assert_eq!(-knowns, board.candidates(cell!("A1")));
        assert_eq!(knowns, board.candidates(cell!("A8")));
        assert_eq!(KnownSet::full(), board.candidates(cell!("B3")));
        assert_eq!(KnownSet::full(), board.candidates(cell!("C2")));
    }

    #[test]
    fn naked_quads() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let knowns = knowns!("1 2 3 4 5");
        board.remove_candidates_from_cells(cells!("A1 A2 A5 A8"), knowns, &mut effects);

        find_naked_quads(&board).unwrap().apply_all(&mut board);

        assert_eq!(-knowns, board.candidates(cell!("A1")));
        assert_eq!(-knowns, board.candidates(cell!("A2")));
        assert_eq!(knowns, board.candidates(cell!("A9")));
        assert_eq!(KnownSet::full(), board.candidates(cell!("B3")));
        assert_eq!(KnownSet::full(), board.candidates(cell!("C2")));
    }
}
