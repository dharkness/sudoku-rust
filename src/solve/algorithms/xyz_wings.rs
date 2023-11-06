use super::*;

pub fn find_xyz_wings(board: &Board) -> Option<Effects> {
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

    tri_values.iter().for_each(|cell| {
        // let (k1, k2) = board.candidates(cell).as_pair().unwrap();
        (cell.peers() & bi_values)
            .iter()
            .combinations(2)
            .map(|pair| pair.iter().copied().union_cells())
            .for_each(|pair| {
                let (c1, c2) = pair.as_pair().expect("cell pair");
                let candidates = cell.peers() & c1.peers() & c2.peers();
                if candidates.len() != 2 {
                    // degenerate naked triple
                    return;
                }

                let ks = board.candidates(cell);
                let ks1 = board.candidates(c1);
                let ks2 = board.candidates(c2);
                if ks1 | ks2 != ks {
                    // degenerate naked pair or totally unrelated candidates
                    return;
                }

                let k = (ks1 & ks2).as_single().expect("one candidate in common");
                if log {
                    println!(
                        "{}-{}: {}-{} {}-{} - {}",
                        cell, ks, c1, ks1, c2, ks2, candidates
                    )
                }

                let mut action = Action::new(Strategy::XYZWing);
                action.erase_cells(candidates & board.candidate_cells(k), k);
                effects.add_action(action);
            });
    });

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}
