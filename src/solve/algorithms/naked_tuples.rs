use super::*;

pub fn find_naked_pairs(board: &Board, single: bool) -> Option<Effects> {
    find_naked_tuples(board, single, 2, Strategy::NakedPair)
}

pub fn find_naked_triples(board: &Board, single: bool) -> Option<Effects> {
    find_naked_tuples(board, single, 3, Strategy::NakedTriple)
}

pub fn find_naked_quads(board: &Board, single: bool) -> Option<Effects> {
    find_naked_tuples(board, single, 4, Strategy::NakedQuad)
}

fn find_naked_tuples(
    board: &Board,
    single: bool,
    size: usize,
    strategy: Strategy,
) -> Option<Effects> {
    let mut effects = Effects::new();

    for house in House::iter() {
        let house_cells = house.cells();
        for candidates in house_cells
            .iter()
            .map(|cell| (cell, board.candidates(cell)))
            .filter(|(_, candidates)| (2..=size).contains(&candidates.len()))
            .combinations(size)
        {
            let known_sets = candidates.iter().map(|(_, ks)| *ks).collect_vec();
            let tuple_knowns = known_sets.iter().copied().union_knowns();
            if tuple_knowns.len() != size
                || is_degenerate(&known_sets, size, 2)
                || is_degenerate(&known_sets, size, 3)
            {
                continue;
            }

            let tuple_cells = candidates.iter().map(|(c, _)| *c).union_cells();
            let erase_cells = house_cells - tuple_cells;
            let mut action = Action::new(strategy);

            tuple_knowns.iter().for_each(|k| {
                action.erase_cells(erase_cells & board.candidate_cells(k), k);
                action.clue_cells_for_known(
                    Verdict::Secondary,
                    tuple_cells & board.candidate_cells(k),
                    k,
                );
            });
            tuple_cells.iter().for_each(|c| {
                action.clue_cell_for_knowns(
                    Verdict::Related,
                    c,
                    KnownSet::full() - board.candidates(c),
                );
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

pub fn is_degenerate(known_sets: &[KnownSet], size: usize, smaller_size: usize) -> bool {
    size > smaller_size
        && known_sets
            .iter()
            .combinations(smaller_size)
            .map(|sets| sets.into_iter().copied().union_knowns())
            .any(|set| (set.len()) <= smaller_size)
}

#[cfg(test)]
mod tests {
    use crate::layout::cells::cell::cell;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::values::known_set::knowns;

    use super::*;

    #[test]
    fn naked_pairs() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let knowns = knowns!("1 2 3 4 5 6 7");
        board.remove_candidates_from_cells(cells!("A1 A2"), knowns, &mut effects);

        find_naked_pairs(&board, false)
            .unwrap()
            .apply_all(&mut board);

        assert_eq!(!knowns, board.candidates(cell!("A1")));
        assert_eq!(!knowns, board.candidates(cell!("A2")));
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

        find_naked_triples(&board, false)
            .unwrap()
            .apply_all(&mut board);

        assert_eq!(!knowns, board.candidates(cell!("A1")));
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

        find_naked_quads(&board, false)
            .unwrap()
            .apply_all(&mut board);

        assert_eq!(!knowns, board.candidates(cell!("A1")));
        assert_eq!(!knowns, board.candidates(cell!("A2")));
        assert_eq!(knowns, board.candidates(cell!("A9")));
        assert_eq!(KnownSet::full(), board.candidates(cell!("B3")));
        assert_eq!(KnownSet::full(), board.candidates(cell!("C2")));
    }
}
