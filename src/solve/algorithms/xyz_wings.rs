use super::*;

pub fn find_xyz_wings(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    // for each tri-value cell
    //   peers = bi-value cells & cell peers
    //   for each pair of peers
    //     if count of cells that see all three cells is not 2
    //       degenerate naked triple (continue)
    //     if union of peer candidates is not cell candidates
    //       continue
    //     found xyz-wing

    let log = false;

    let tri_values = board.cells_with_n_candidates(3);
    if tri_values.is_empty() {
        return None;
    }
    let bi_values = board.cells_with_n_candidates(2);
    if bi_values.is_empty() {
        return None;
    }

    for pivot in tri_values {
        // let (k1, k2) = board.candidates(cell).as_pair().unwrap();
        let pivot_peers = pivot.peers();
        for pair in (pivot_peers & bi_values)
            .iter()
            .combinations(2)
            .map(|pair| pair.iter().copied().union_cells())
        {
            let (c1, c2) = pair.as_pair().expect("cell pair");
            let candidates = pivot_peers & c1.peers() & c2.peers();
            if candidates.len() != 2 {
                // degenerate naked triple
                continue;
            }

            let ks = board.candidates(pivot);
            let ks1 = board.candidates(c1);
            let ks2 = board.candidates(c2);
            if ks1 | ks2 != ks {
                // degenerate naked pair or totally unrelated candidates
                continue;
            }

            let k = (ks1 & ks2).as_single().expect("one candidate in common");
            if log {
                println!(
                    "{}-{}: {}-{} {}-{} - {}",
                    pivot, ks, c1, ks1, c2, ks2, candidates
                )
            }

            let mut action = Action::new(Strategy::XYZWing);
            action.erase_cells(candidates & board.candidate_cells(k), k);
            action.clue_cells_for_known(Verdict::Secondary, pair + pivot, k);
            action.clue_cell_for_knowns(Verdict::Primary, pivot, ks1 - k);
            action.clue_cell_for_knowns(Verdict::Primary, pivot, ks2 - k);
            action.clue_cell_for_knowns(Verdict::Primary, c1, ks1 - k);
            action.clue_cell_for_knowns(Verdict::Primary, c2, ks2 - k);

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

#[cfg(test)]
mod tests {
    use crate::io::{Parse, Parser};
    use crate::layout::cells::cell::cell;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::values::known::known;

    use super::*;

    #[test]
    fn all_in_pivot() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "88g1052g8g0341112o110q6i054gm02og2812i8i6ijg09v005g22g8841110a860hg1210605880iga21o08i41110i21g14116948i090m2i1i813ig1703a056a411m2i3i1m093281g1g11609815674320h62",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_xyz_wings(&board, true) {
            assert_eq!(1, got.actions().len());

            let mut action = Action::new(Strategy::XYZWing);
            action.erase(cell!("F7"), known!("1"));
            action.clue_cells_for_known(Verdict::Secondary, cells!("D9 F1 F9"), known!("1"));
            action.clue_cells_for_known(Verdict::Primary, cells!("D9 F9"), known!("2"));
            action.clue_cells_for_known(Verdict::Primary, cells!("F1 F9"), known!("4"));

            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }
}
