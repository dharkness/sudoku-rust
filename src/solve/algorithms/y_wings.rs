use super::*;

pub fn find_y_wings(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    let bi_values = board.cells_with_n_candidates(2);
    let log = false;

    for pivot in bi_values {
        let (k1, k2) = board.candidates(pivot).as_pair().unwrap();
        let peers = pivot.peers() & bi_values;
        if peers.len() < 2 {
            continue;
        }

        let k1_peers = peers & board.candidate_cells(k1);
        let k2_peers = peers & board.candidate_cells(k2);

        if log {
            println!("{}: {}-{}: {}-{}", pivot, k1, k2, k1_peers, k2_peers)
        }

        for c1 in k1_peers {
            let k1_other = board.candidates(c1) - k1;
            for c2 in k2_peers {
                let k2_other = board.candidates(c2) - k2;
                if k1_other != k2_other || c1.sees(c2) {
                    continue;
                }

                let k = k1_other.iter().next().unwrap();
                let erase = c1.peers() & c2.peers() & board.candidate_cells(k);
                if erase.is_empty() {
                    continue;
                }

                let mut action = Action::new(Strategy::YWing);
                action.erase_cells(erase, k);
                action.clue_cell_for_known(Verdict::Secondary, pivot, k1);
                action.clue_cell_for_known(Verdict::Tertiary, pivot, k2);
                action.clue_cell_for_known(Verdict::Tertiary, c1, k1);
                action.clue_cell_for_known(Verdict::Secondary, c1, k);
                action.clue_cell_for_known(Verdict::Secondary, c2, k2);
                action.clue_cell_for_known(Verdict::Tertiary, c2, k);

                if effects.add_action(action) && single {
                    return Some(effects);
                }
            }
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}
