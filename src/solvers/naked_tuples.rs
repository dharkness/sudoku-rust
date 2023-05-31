use crate::layout::{Cell, CellSet, House, KnownSet};
use crate::puzzle::{Action, Board, Effects, Strategy};

use super::distinct_tuples::*;

type CellCandidates = (Cell, KnownSet);

pub fn find_naked_pairs(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    for house in House::all() {
        let cell_candidates: Vec<CellCandidates> = house
            .cells()
            .iter()
            .map(|cell| (cell, board.candidates(cell)))
            .filter(|(_, candidates)| candidates.size() == 2)
            .collect::<Vec<_>>();

        for candidates in distinct_pairs(&cell_candidates) {
            let known_sets = vec![candidates.0 .1, candidates.1 .1];
            let knowns = known_sets
                .iter()
                .fold(KnownSet::empty(), |acc, ks| acc | *ks);
            if knowns.size() != 2 {
                continue;
            }

            let cells = house.cells() - candidates.0 .0 - candidates.1 .0;
            erase_knowns_from_cells(board, cells, knowns, Strategy::NakedPair, &mut effects);
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

pub fn find_naked_triples(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    for house in House::all() {
        let cell_candidates: Vec<CellCandidates> = house
            .cells()
            .iter()
            .map(|cell| (cell, board.candidates(cell)))
            .filter(|(_, candidates)| candidates.size() == 2 || candidates.size() == 3)
            .collect::<Vec<_>>();

        for candidates in distinct_triples(&cell_candidates) {
            let known_sets = vec![candidates.0 .1, candidates.1 .1, candidates.2 .1];
            let knowns = known_sets
                .iter()
                .fold(KnownSet::empty(), |acc, ks| acc | *ks);
            if knowns.size() != 3 {
                continue;
            }
            if distinct_pairs(&known_sets)
                .iter()
                .any(|(ks1, ks2)| (*ks1 | *ks2).size() < 3)
            {
                continue;
            }

            let cells = house.cells() - candidates.0 .0 - candidates.1 .0 - candidates.2 .0;
            erase_knowns_from_cells(board, cells, knowns, Strategy::NakedPair, &mut effects);
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

pub fn find_naked_quads(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    for house in House::all() {
        let cell_candidates: Vec<CellCandidates> = house
            .cells()
            .iter()
            .map(|cell| (cell, board.candidates(cell)))
            .filter(|(_, candidates)| {
                candidates.size() == 2 || candidates.size() == 3 || candidates.size() == 4
            })
            .collect::<Vec<_>>();

        for candidates in distinct_quads(&cell_candidates) {
            let known_sets = vec![
                candidates.0 .1,
                candidates.1 .1,
                candidates.2 .1,
                candidates.3 .1,
            ];
            let knowns = known_sets
                .iter()
                .fold(KnownSet::empty(), |acc, ks| acc | *ks);
            if knowns.size() != 4 {
                continue;
            }
            if distinct_pairs(&known_sets)
                .iter()
                .any(|(ks1, ks2)| (*ks1 | *ks2).size() < 3)
            {
                continue;
            }
            if distinct_triples(&known_sets)
                .iter()
                .any(|(ks1, ks2, ks3)| (*ks1 | *ks2 | *ks3).size() < 4)
            {
                continue;
            }

            let cells = house.cells()
                - candidates.0 .0
                - candidates.1 .0
                - candidates.2 .0
                - candidates.3 .0;
            erase_knowns_from_cells(board, cells, knowns, Strategy::NakedPair, &mut effects);
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

fn erase_knowns_from_cells(
    board: &Board,
    cells: CellSet,
    knowns: KnownSet,
    strategy: Strategy,
    effects: &mut Effects,
) {
    let mut action = Action::new(strategy);

    knowns
        .iter()
        .for_each(|k| action.erase_cells(cells & board.candidate_cells(k), k));

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

    #[test]
    fn naked_pairs() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let knowns = knowns!("1 2 3 4 5 6 7");
        board.remove_many_candidates(cells!("A1 A2"), knowns, &mut effects);

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
        board.remove_many_candidates(cells!("A1 A2 A5"), knowns, &mut effects);

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
        board.remove_many_candidates(cells!("A1 A2 A5 A8"), knowns, &mut effects);

        find_naked_quads(&board).unwrap().apply_all(&mut board);

        assert_eq!(-knowns, board.candidates(cell!("A1")));
        assert_eq!(-knowns, board.candidates(cell!("A2")));
        assert_eq!(knowns, board.candidates(cell!("A9")));
        assert_eq!(KnownSet::full(), board.candidates(cell!("B3")));
        assert_eq!(KnownSet::full(), board.candidates(cell!("C2")));
    }
}
