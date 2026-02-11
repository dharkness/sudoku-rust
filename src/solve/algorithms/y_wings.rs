use super::*;

pub fn find_y_wings(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    let bi_values = board.cells_with_n_candidates(2);
    let log = false;

    for pivot in bi_values {
        let (d1, d2) = board.candidates(pivot).as_pair().unwrap();
        let peers = pivot.peers() & bi_values;
        if peers.len() < 2 {
            continue;
        }

        let k1_peers = peers & board.candidate_cells(d1);
        let k2_peers = peers & board.candidate_cells(d2);

        if log {
            println!("{}: {}-{}: {}-{}", pivot, d1, d2, k1_peers, k2_peers)
        }

        for c1 in k1_peers {
            let k1_other = board.candidates(c1) - d1;
            for c2 in k2_peers {
                let k2_other = board.candidates(c2) - d2;
                if k1_other != k2_other || c1.sees(c2) {
                    continue;
                }

                let d = k1_other.iter().next().unwrap();
                let erase = c1.peers() & c2.peers() & board.candidate_cells(d);
                if erase.is_empty() {
                    continue;
                }

                let mut action = Action::new(Strategy::YWing);
                action.erase_cells(erase, d);
                action.clue_cell_for_digit(Verdict::Secondary, pivot, d1);
                action.clue_cell_for_digit(Verdict::Tertiary, pivot, d2);
                action.clue_cell_for_digit(Verdict::Tertiary, c1, d1);
                action.clue_cell_for_digit(Verdict::Secondary, c1, d);
                action.clue_cell_for_digit(Verdict::Secondary, c2, d2);
                action.clue_cell_for_digit(Verdict::Tertiary, c2, d);

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
