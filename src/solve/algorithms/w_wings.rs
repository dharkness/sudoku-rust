use std::collections::{HashMap, VecDeque};

use super::*;

pub fn find_w_wings(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    let mut pairs_by_candidates: HashMap<DigitSet, CellSet> = HashMap::new();
    for (cell, candidates) in board.cells_with_n_candidates_iter(2) {
        *pairs_by_candidates.entry(candidates).or_default() += cell;
    }

    let strong_links = board.strong_links();

    for (pair, cells) in pairs_by_candidates {
        let Some((d1, d2)) = pair.as_pair() else {
            continue;
        };

        if add_single_w_wings(
            board,
            pair,
            cells,
            d1,
            d2,
            &strong_links,
            &mut effects,
            single,
        ) {
            return Some(effects);
        }

        if add_remote_pair_chains(board, pair, cells, &strong_links, &mut effects, single) {
            return Some(effects);
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

fn add_single_w_wings(
    board: &Board,
    pair: DigitSet,
    cells: CellSet,
    d1: Digit,
    d2: Digit,
    strong_links: &[PeerSet; 9],
    effects: &mut Effects,
    single: bool,
) -> bool {
    let cell_pairs = cells.iter().combinations(2).collect_vec();

    for (link_digit, erase_digit) in [(d1, d2), (d2, d1)] {
        let links = &strong_links[link_digit.usize()];
        if links.is_empty() {
            continue;
        }

        for combo in &cell_pairs {
            let c1 = combo[0];
            let c2 = combo[1];
            if c1.sees(c2) {
                continue;
            }

            for (s1, s2) in links.iter() {
                let sees_direct = c1.sees(s1) && c2.sees(s2);
                let sees_cross = c1.sees(s2) && c2.sees(s1);
                if !(sees_direct || sees_cross) {
                    continue;
                }

                let erase = c1.peers() & c2.peers() & board.candidate_cells(erase_digit);
                if erase.is_empty() {
                    continue;
                }

                let mut action = Action::new(Strategy::WWing);
                action.erase_cells(erase, erase_digit);
                action.clue_cells_for_digits(Verdict::Secondary, CellSet::of(&[c1, c2]), pair);
                action.clue_cells_for_digit(Verdict::Primary, CellSet::of(&[s1, s2]), link_digit);

                if effects.add_action(action) && single {
                    return true;
                }
            }
        }
    }

    false
}

fn add_remote_pair_chains(
    board: &Board,
    pair: DigitSet,
    cells: CellSet,
    strong_links: &[PeerSet; 9],
    effects: &mut Effects,
    single: bool,
) -> bool {
    if cells.len() < 4 {
        return false;
    }

    let mut adjacency: HashMap<Cell, CellSet> = HashMap::new();
    for digit in pair {
        for (a, b) in &strong_links[digit.usize()] {
            if cells.has(a) && cells.has(b) {
                adjacency.entry(a).or_default().add(b);
                adjacency.entry(b).or_default().add(a);
            }
        }
    }
    if adjacency.is_empty() {
        return false;
    }

    let mut node_info: HashMap<Cell, (usize, bool)> = HashMap::new();
    let mut conflicts: Vec<bool> = Vec::new();

    for cell in cells {
        if node_info.contains_key(&cell) {
            continue;
        }

        conflicts.push(false);
        let component = conflicts.len() - 1;
        let mut queue = VecDeque::new();
        node_info.insert(cell, (component, false));
        queue.push_back(cell);

        while let Some(current) = queue.pop_front() {
            let (_, parity) = node_info[&current];
            let Some(neighbors) = adjacency.get(&current) else {
                continue;
            };
            for neighbor in *neighbors {
                if let Some((_, seen_parity)) = node_info.get(&neighbor) {
                    if *seen_parity == parity {
                        conflicts[component] = true;
                    }
                } else {
                    node_info.insert(neighbor, (component, !parity));
                    queue.push_back(neighbor);
                }
            }
        }
    }

    let cell_pairs = cells.iter().combinations(2).collect_vec();
    for combo in cell_pairs {
        let c1 = combo[0];
        let c2 = combo[1];
        if c1.sees(c2) {
            continue;
        }
        let Some((comp1, parity1)) = node_info.get(&c1) else {
            continue;
        };
        let Some((comp2, parity2)) = node_info.get(&c2) else {
            continue;
        };
        if comp1 != comp2 || conflicts[*comp1] || parity1 == parity2 {
            continue;
        }

        let erase_base = c1.peers() & c2.peers();
        let mut action = Action::new(Strategy::WWing);
        for digit in pair {
            let erase = erase_base & board.candidate_cells(digit);
            if !erase.is_empty() {
                action.erase_cells(erase, digit);
            }
        }
        if action.is_empty() {
            continue;
        }
        action.clue_cells_for_digits(Verdict::Secondary, CellSet::of(&[c1, c2]), pair);

        if effects.add_action(action) && single {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use crate::io::{Parse, Parser};
    use crate::*;

    use super::*;

    fn assert_any_erase(actions: &[Action], cell: Cell, digit: Digit) {
        assert!(
            actions.iter().any(|action| action.erases(cell, digit)),
            "expected erase of {} from {}",
            digit,
            cell
        );
    }

    fn assert_any_erase_in(actions: &[Action], targets: CellSet, digit: Digit) {
        assert!(
            actions
                .iter()
                .any(|action| { targets.iter().any(|cell| action.erases(cell, digit)) }),
            "expected erase of {} from {}",
            digit,
            targets
        );
    }

    #[test]
    fn remote_pair_chain_example_1() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B0f017u020c0e08077u0b050g0i0d0h0c0f017ybibi0a07060e7u0b087u121202070a7u068e807w120f0a7u0h0g010g0f04080i0b0e037y0f8008050w070a7u07bgbw0f0a0s7u034i1a4e010g0i0u0f024i",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_w_wings(&board, false).expect("not found");
        let actions = got.actions();
        assert_any_erase(actions, cell!(H3), digit!(4));
        assert_any_erase(actions, cell!(H3), digit!(9));
    }

    #[test]
    fn remote_pair_chain_example_2() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B0f017u020c0e08077u0b050g0i0d0h0c0f017ybibi0a07060e7u0b087u121202070a7u068e807w120f0a7u0h0g010g0f04080i0b0e037y0f8008050w070a7u07bg4k0f0a0s7u034i1a4e010g0i0u0f024i",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_w_wings(&board, false).expect("not found");
        let actions = got.actions();
        assert_any_erase(actions, cell!(H2), digit!(4));
        assert_any_erase(actions, cell!(H2), digit!(9));
    }

    #[test]
    fn single_w_wing_example_1() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B7n0h1m1q021i0z82077o1m0c050a07088k8s0l05071m090h1p031p121m08090g20280128070a0i1k0420201w0812021m0h1i01090g260h090a1k051k07041i0d0g02011i091y0h1y060c0e0g080d0l7o7p",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_w_wings(&board, false).expect("not found");
        let actions = got.actions();
        assert_any_erase(actions, cell!(D6), digit!(3));
        assert_any_erase(actions, cell!(E6), digit!(3));
    }

    #[test]
    fn single_w_wing_example_2() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B7n0h1m1q021i0z82077o1m0c050a07088k8s0l05071m090h1p031p121m08090g1w280128070a0i1k041w201w0812021m0h1i01090g260h090a1k051k07041i0d0g02011i091y0h1y060c0e0g080d0l7o7p",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_w_wings(&board, false).expect("not found");
        let actions = got.actions();
        let targets = cell!(B1).peers() & cell!(J8).peers();
        assert_any_erase_in(actions, targets, digit!(2));
    }

    #[test]
    fn split_w_wing_example() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B0e2i0i0h010c062i0b0c1n1m070b090h0r052b080b0f050d09032b1m053e01030h0b093e0h0c3e0b090e2j373f0i020a0d06070e080c3e09030e080a2i023e020r0h030g060r0e0i3737050i040b0c370h",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_w_wings(&board, false).expect("not found");
        let actions = got.actions();
        assert_any_erase(actions, cell!(B2), digit!(4));
        assert_any_erase(actions, cell!(B2), digit!(1));
    }
}
