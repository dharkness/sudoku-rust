use crate::layout::*;
use crate::puzzle::{Action, Board, Effects, Strategy};
use std::ops::Range;

type CellCandidates = (Cell, KnownSet);

pub fn find_naked_pairs(board: &Board) -> Effects {
    let mut effects = Effects::new();

    for house in House::all() {
        let cell_candidates: Vec<CellCandidates> = house
            .cells()
            .iter()
            .map(|cell| (cell, board.candidates(cell)))
            .filter(|(_, candidates)| candidates.size() == 2)
            .collect::<Vec<_>>();

        for candidates in distinct_pairs(&cell_candidates) {
            let knowns = [candidates.0 .1, candidates.1 .1];
            let union = knowns[0] | knowns[1];
            if union.size() != 2 {
                continue;
            }

            let cells = CellSet::empty() + candidates.0 .0 + candidates.1 .0;
            add_action_to_erase_knowns(
                board,
                house,
                cells,
                union,
                Strategy::NakedPair,
                &mut effects,
            );
        }
    }

    effects
}

pub fn find_naked_triples(board: &Board) -> Effects {
    let mut effects = Effects::new();

    for house in House::all() {
        let cell_candidates: Vec<CellCandidates> = house
            .cells()
            .iter()
            .map(|cell| (cell, board.candidates(cell)))
            .filter(|(_, candidates)| candidates.size() == 2 || candidates.size() == 3)
            .collect::<Vec<_>>();

        for candidates in distinct_triples(&cell_candidates) {
            let knowns = vec![candidates.0 .1, candidates.1 .1, candidates.2 .1];
            let union = knowns.iter().fold(KnownSet::empty(), |acc, ks| acc | *ks);
            if union.size() != 3 {
                continue;
            }
            if distinct_pairs(&knowns)
                .iter()
                .any(|(ks1, ks2)| (*ks1 | *ks2).size() < 3)
            {
                continue;
            }

            let cells = CellSet::empty() + candidates.0 .0 + candidates.1 .0 + candidates.2 .0;
            add_action_to_erase_knowns(
                board,
                house,
                cells,
                union,
                Strategy::NakedPair,
                &mut effects,
            );
        }
    }

    effects
}

pub fn find_naked_quads(board: &Board) -> Effects {
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
            let knowns = vec![
                candidates.0 .1,
                candidates.1 .1,
                candidates.2 .1,
                candidates.3 .1,
            ];
            let union = knowns.iter().fold(KnownSet::empty(), |acc, ks| acc | *ks);
            if union.size() != 4 {
                continue;
            }
            if distinct_pairs(&knowns)
                .iter()
                .any(|(ks1, ks2)| (*ks1 | *ks2).size() < 3)
            {
                continue;
            }
            if distinct_triples(&knowns)
                .iter()
                .any(|(ks1, ks2, ks3)| (*ks1 | *ks2 | *ks3).size() < 4)
            {
                continue;
            }

            let cells = CellSet::empty()
                + candidates.0 .0
                + candidates.1 .0
                + candidates.2 .0
                + candidates.3 .0;
            add_action_to_erase_knowns(
                board,
                house,
                cells,
                union,
                Strategy::NakedPair,
                &mut effects,
            );
        }
    }

    effects
}

fn add_action_to_erase_knowns(
    board: &Board,
    house: &House,
    cells: CellSet,
    knowns: KnownSet,
    strategy: Strategy,
    effects: &mut Effects,
) {
    let mut action = Action::new(strategy);
    for k in knowns.iter() {
        let diff = house.cells() & board.candidate_cells(k) - cells;
        if !diff.is_empty() {
            action.erase_cells(diff, k);
        }
    }

    if !action.is_empty() {
        effects.add_action(action);
    }
}

fn distinct_pairs<T: Copy>(items: &Vec<T>) -> Vec<(T, T)> {
    let mut pairs = Vec::new();
    for i in 0..items.len() {
        for j in i + 1..items.len() {
            pairs.push((items[i], items[j]));
        }
    }
    pairs
}

fn distinct_triples<T: Copy>(items: &Vec<T>) -> Vec<(T, T, T)> {
    let mut pairs = Vec::new();
    for i in 0..items.len() {
        for j in i + 1..items.len() {
            for k in j + 1..items.len() {
                pairs.push((items[i], items[j], items[k]));
            }
        }
    }
    pairs
}

fn distinct_quads<T: Copy>(items: &Vec<T>) -> Vec<(T, T, T, T)> {
    let mut pairs = Vec::new();
    for i in 0..items.len() {
        for j in i + 1..items.len() {
            for k in j + 1..items.len() {
                for l in k + 1..items.len() {
                    pairs.push((items[i], items[j], items[k], items[l]));
                }
            }
        }
    }
    pairs
}
