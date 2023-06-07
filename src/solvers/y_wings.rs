use super::*;

pub fn find_y_wings(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    let bi_values = board.cells_with_n_candidates(2);
    let mut log = false;

    println!("bi_values: {}", bi_values);

    bi_values.iter().for_each(|cell| {
        let (k1, k2) = board.candidates(cell).as_pair().unwrap();
        let peers = cell.peers() & bi_values;
        if peers.size() < 2 {
            return;
        }

        log = cell.label() == "A2";

        let k1_peers = peers & board.candidate_cells(k1);
        let k2_peers = peers & board.candidate_cells(k2);

        if log {
            println!("{}: {}-{}: {}-{}", cell, k1, k2, k1_peers, k2_peers)
        }

        k1_peers.iter().for_each(|c1| {
            let k1_other = board.candidates(c1) - k1;
            k2_peers.iter().for_each(|c2| {
                let k2_other = board.candidates(c2) - k2;
                if k1_other != k2_other || c1.sees(c2) {
                    return;
                }

                let k = k1_other.iter().next().unwrap();
                let erase = c1.peers() & c2.peers() & board.candidate_cells(k);
                if erase.is_empty() {
                    return;
                }

                let mut action = Action::new(Strategy::YWing);
                action.erase_cells(erase, k);
                effects.add_action(action);
            });
        });
    });

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}
