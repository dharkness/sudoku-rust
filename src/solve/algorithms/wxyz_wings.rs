use std::collections::HashMap;

use super::*;

pub fn find_wxyz_wings(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    let pairs_by_candidates = board.cells_with_n_candidates_iter(2).fold(
        HashMap::new(),
        |mut map: HashMap<DigitSet, CellSet>, (cell, candidates)| {
            *map.entry(candidates).or_default() += cell;
            map
        },
    );
    let triples_by_candidates = board.cells_with_n_candidates_iter(3).fold(
        HashMap::new(),
        |mut map: HashMap<DigitSet, CellSet>, (cell, candidates)| {
            *map.entry(candidates).or_default() += cell;
            map
        },
    );
    let quads_by_candidates = board.cells_with_n_candidates_iter(4).fold(
        HashMap::new(),
        |mut map: HashMap<DigitSet, CellSet>, (cell, candidates)| {
            *map.entry(candidates).or_default() += cell;
            map
        },
    );

    // match 1-4 quad cells with any mix of pairs and triples
    let quad_sets = quads_by_candidates
        .iter()
        .map(|(candidates, cells)| {
            (
                *candidates,
                *cells,
                // pairs and triples with all candidates
                triples_by_candidates
                    .iter()
                    .filter(|(c, _)| c.is_subset_of(*candidates))
                    .map(|(_, cells)| *cells)
                    .union_cells()
                    | pairs_by_candidates
                        .iter()
                        .filter(|(c, _)| c.is_subset_of(*candidates))
                        .map(|(_, cells)| *cells)
                        .union_cells(),
            )
        })
        .collect_vec();

    // match 1-4 triple cells with pairs and triples, grouped by the disjoint candidates
    // e.g. a 123 triple can mix with pairs and triples with one fewer digit as long as
    // they all share the same extra digit: 14, 24, and 34 pairs with 124, 134 triples, etc.
    let triple_sets = triples_by_candidates
        .iter()
        .map(|(candidates, cells)| {
            let triples_with_two_common_candidates =
                triples_by_candidates
                    .iter()
                    .fold(HashMap::new(), |mut acc, (ds, cs)| {
                        let diff = *ds - *candidates;
                        if let Some(single) = diff.as_single() {
                            *acc.entry(single).or_insert_with(CellSet::empty) |= *cs;
                        }
                        acc
                    });
            (
                *candidates,
                *cells,
                // pairs and triples with one fewer candidate, grouped by the extra candidate
                pairs_by_candidates.iter().fold(
                    triples_with_two_common_candidates,
                    |mut acc, (ds, cs)| {
                        let diff = *ds - *candidates;
                        if let Some(single) = diff.as_single() {
                            *acc.entry(single).or_insert_with(CellSet::empty) |= *cs;
                        }
                        acc
                    },
                ),
                pairs_by_candidates
                    .iter()
                    .filter(|(ds, _)| ds.is_subset_of(*candidates))
                    .map(|(_, cells)| *cells)
                    .union_cells(),
            )
        })
        .collect_vec();

    // the other bi-value cells that each bi-value cell sees with the same two candidates
    // only tracks the earlier of the two cells
    //
    // TODO iterate candidates c, intersect candidates with c.peers(), remove cells < c, and store the result as the seen bi-value cells for c
    let seen_bi_values: HashMap<Cell, CellSet> =
        pairs_by_candidates
            .iter()
            .fold(HashMap::new(), |mut map, (_, cells)| {
                for (c1, c2) in cells.pair_iter() {
                    if c1.sees(c2) {
                        *map.entry(c1).or_default() += c2;
                    }
                }
                map
            });

    let bi_values = board.cells_with_n_candidates(2);
    let mut check_wing = |wing: CellSet| -> bool {
        // println!("wing {}", wing);
        // ignore xy chains
        if (wing & bi_values) == wing {
            return false;
        }
        // ignore naked quads
        if wing.share_any_house() {
            return false;
        }
        // ignore naked pairs
        if (wing & bi_values).iter().any(|cell| {
            if let Some(seen) = seen_bi_values.get(&cell) {
                if !(*seen & wing).is_empty() {
                    return true;
                }
            }
            false
        }) {
            return false;
        }

        let wing_digits = wing
            .iter()
            .fold(DigitSet::empty(), |set, cell| set | board.candidates(cell));
        if wing_digits.len() != 4 {
            return false;
        }
        if wing_digits
            .iter()
            .any(|digit| (wing & board.candidate_cells(digit)).len() < 2)
        {
            return false;
        }

        let mut restricted: HashMap<Digit, CellSet> = HashMap::new();
        let mut non_restricted: HashMap<Digit, CellSet> = HashMap::new();
        for digit in wing_digits {
            let candidates = wing & board.candidate_cells(digit);
            if candidates.pair_iter().all(|(c1, c2)| c1.sees(c2)) {
                restricted.insert(digit, candidates);
            } else {
                if !non_restricted.is_empty() {
                    return false;
                }
                non_restricted.insert(digit, candidates);
            }
        }
        if non_restricted.is_empty() {
            return false;
        }

        let (candidate, cells) = non_restricted.into_iter().next().unwrap();
        let erase = cells
            .iter()
            .fold(board.candidate_cells(candidate), |set, cell| {
                set & cell.peers()
            });
        if erase.is_empty() {
            return false;
        }

        let mut action = Action::new_erase_cells(Strategy::WXYZWing, erase, candidate);
        action.clue_cells_for_digit(Verdict::Secondary, cells, candidate);
        for (digit, cells) in restricted {
            action.clue_cells_for_digit(Verdict::Primary, cells, digit);
        }

        effects.add_action(action)
    };

    for (_, quads, subsets) in quad_sets {
        // 4 quads
        for quad_combo in quads.iter().combinations(4) {
            if check_wing(quad_combo.iter().copied().union_cells()) && single {
                return Some(effects);
            }
        }
        // 2..3 quads with 4-n triples and pairs
        for n in (2..4).rev() {
            for quad_combo in quads.iter().combinations(n) {
                let base = quad_combo.iter().copied().union_cells();
                for others in subsets.iter().combinations(4 - n) {
                    if check_wing(base | others.iter().copied().union_cells()) && single {
                        return Some(effects);
                    }
                }
            }
        }
        // 1 quad with 3 triples and pairs
        for quad in quads {
            for others in subsets.iter().combinations(3) {
                if check_wing(others.iter().copied().union_cells() + quad) && single {
                    return Some(effects);
                }
            }
        }
    }

    for (candidates, triples, disjoints, subsets) in triple_sets {
        // 4 primary triples
        for triple_combo in triples.iter().combinations(4) {
            if check_wing(triple_combo.iter().copied().union_cells()) && single {
                return Some(effects);
            }
        }
        // 1..3 primary triples with 4-n secondary triples and pairs
        for n in (1..4).rev() {
            for triple_combo in triples.iter().combinations(n) {
                let base = triple_combo.iter().copied().union_cells();
                for d in !candidates {
                    if let Some(disjoint) = disjoints.get(&d) {
                        for others in (*disjoint | subsets).iter().combinations(4 - n) {
                            if check_wing(base | others.iter().copied().union_cells()) && single {
                                return Some(effects);
                            }
                        }
                    }
                }
            }
        }

        // future improvement: split these out from above
        // 1..3 primary triples with 4-n pairs
        // 1..2 primary triples with 1..3-n secondary triples and rest as pairs
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::io::{Parse, Parser};
    use crate::*;

    use super::*;

    #[test]
    fn all_in_pivot() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "q2i2o2p2050h410992110ho20941o0052182050941b212b0h2o20h41j6h6h009h4810h32i0j4090h8103h0k470g2810hl021l4h20609090686e20ie02g11g18i4190b2g1092g8205oih221051i9009c2c2",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_wxyz_wings(&board, true) {
            assert_eq!(1, got.actions().len());

            let mut action = Action::new(Strategy::WXYZWing);
            action.erase(cell!(D2), digit!(9));
            action.clue_cells_for_digit(Verdict::Secondary, cells![D3 D4 D6 F1], digit!(9));
            action.clue_cells_for_digit(Verdict::Primary, cells![D3 F1], digit!(1));
            action.clue_cells_for_digit(Verdict::Primary, cells![D3 D6], digit!(2));
            action.clue_cells_for_digit(Verdict::Primary, cells![D3 D4 D6], digit!(5));

            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }

    #[test]
    fn not_all_in_pivot() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "810h053030094103g160m00903s00hb0942411m00344s4o4a0090hi009812403i40h4111gg051g09h0410321810330413g9gb005g1090990215k5k14g19g034gd01gg10903b094240503g1813g30091g41",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_wxyz_wings(&board, true) {
            assert_eq!(1, got.actions().len());

            let mut action = Action::new(Strategy::WXYZWing);
            action.erase_cells(cells![F6 G5 J5], digit!(5));
            action.clue_cells_for_digit(Verdict::Secondary, cells![E5 G6 J6], digit!(5));
            action.clue_cells_for_digit(Verdict::Primary, cells![D6 J6], digit!(6));
            action.clue_cells_for_digit(Verdict::Primary, cells![D6 G6], digit!(2));
            action.clue_cells_for_digit(Verdict::Primary, cells![D6 E5], digit!(9));

            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }

    #[test]
    fn ignores_naked_pairs() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "4m8111kcka06gk21gk06i6i6j4o20h4108p44k09m4n4s0b403okpk8e0goem0k222o411u621h6o6l00h09o4o6s61241g281053209giii8e068e0h11g12141065232620c8884hggigig1140h032140948409",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        assert_eq!(None, find_wxyz_wings(&board, true));
    }

    #[test]
    fn each_primary_must_appear_at_least_twice() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "k020081102k0800h0503s0d4g4210gl008h00gk014084481h02002210gk0g4441003810804s2s22009k20g10k01008k20h80k204k02008050h41h020h0028080112002gg0409k0kgk0k2k2801g0821041g"
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        assert_eq!(None, find_wxyz_wings(&board, true));
    }

    #[test]
    fn ignores_xy_chains() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "0305090h5050a0a0g10h11210381g10541094181g10921050h1103054i4i50522109g181094281g10h4211242421g11105098141030h114a42e0k64aq20h24814q0560k24qi22811g1210i90161q828c41"
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        assert_eq!(None, find_wxyz_wings(&board, true));
    }
}
