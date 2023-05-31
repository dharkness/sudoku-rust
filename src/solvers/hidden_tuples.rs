use crate::layout::{CellSet, House, Known, KnownSet};
use crate::puzzle::{Action, Board, Effects, Strategy};

use super::distinct_tuples::*;

type KnownCandidates = (Known, CellSet);

pub fn find_hidden_pairs(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    for house in House::all() {
        let cell_candidates: Vec<KnownCandidates> = Known::ALL
            .into_iter()
            .map(|k| (k, house.cells() & board.candidate_cells(k)))
            .filter(|(_, candidates)| candidates.size() == 2)
            .collect::<Vec<_>>();

        for candidates in distinct_pairs(&cell_candidates) {
            let cell_sets = vec![candidates.0 .1, candidates.1 .1];
            let cells = cell_sets.iter().fold(CellSet::empty(), |acc, cs| acc | *cs);
            if cells.size() != 2 {
                continue;
            }

            let knowns = candidates.0 .0 + candidates.1 .0;
            erase_other_knowns_from_cells(board, cells, knowns, Strategy::NakedPair, &mut effects);
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

pub fn find_hidden_triples(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    for house in House::all() {
        let cell_candidates: Vec<KnownCandidates> = Known::ALL
            .into_iter()
            .map(|k| (k, house.cells() & board.candidate_cells(k)))
            .filter(|(_, candidates)| candidates.size() == 2 || candidates.size() == 3)
            .collect::<Vec<_>>();

        for candidates in distinct_triples(&cell_candidates) {
            let cell_sets = vec![candidates.0 .1, candidates.1 .1, candidates.2 .1];
            let cells = cell_sets.iter().fold(CellSet::empty(), |acc, cs| acc | *cs);
            if cells.size() != 3 {
                continue;
            }
            if distinct_pairs(&cell_sets)
                .iter()
                .any(|(cs1, cs2)| (*cs1 | *cs2).size() < 3)
            {
                continue;
            }

            let knowns = candidates.0 .0 + candidates.1 .0 + candidates.2 .0;
            erase_other_knowns_from_cells(board, cells, knowns, Strategy::NakedPair, &mut effects);
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

pub fn find_hidden_quads(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    for house in House::all() {
        let cell_candidates: Vec<KnownCandidates> = Known::ALL
            .into_iter()
            .map(|k| (k, house.cells() & board.candidate_cells(k)))
            .filter(|(_, candidates)| {
                candidates.size() == 2 || candidates.size() == 3 || candidates.size() == 4
            })
            .collect::<Vec<_>>();

        for candidates in distinct_quads(&cell_candidates) {
            let cell_sets = vec![
                candidates.0 .1,
                candidates.1 .1,
                candidates.2 .1,
                candidates.3 .1,
            ];
            let cells = cell_sets.iter().fold(CellSet::empty(), |acc, cs| acc | *cs);
            if cells.size() != 4 {
                continue;
            }
            if distinct_pairs(&cell_sets)
                .iter()
                .any(|(cs1, cs2)| (*cs1 | *cs2).size() < 3)
            {
                continue;
            }
            if distinct_triples(&cell_sets)
                .iter()
                .any(|(cs1, cs2, cs3)| (*cs1 | *cs2 | *cs3).size() < 4)
            {
                continue;
            }

            let knowns = candidates.0 .0 + candidates.1 .0 + candidates.2 .0 + candidates.3 .0;
            erase_other_knowns_from_cells(board, cells, knowns, Strategy::NakedPair, &mut effects);
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

fn erase_other_knowns_from_cells(
    board: &Board,
    cells: CellSet,
    except: KnownSet,
    strategy: Strategy,
    effects: &mut Effects,
) {
    let mut action = Action::new(strategy);

    cells
        .iter()
        .for_each(|c| action.erase_knowns(c, board.candidates(c) - except));

    if !action.is_empty() {
        effects.add_action(action);
    }
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
        board.remove_many_candidates(cells, knowns, &mut effects);

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
        board.remove_many_candidates(cells, knowns, &mut effects);

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
        board.remove_many_candidates(cells, knowns, &mut effects);

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
