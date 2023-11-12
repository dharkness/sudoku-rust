use super::*;

pub fn find_peers(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    for (cell, known) in board.known_iter() {
        let peers = cell.peers() & board.candidate_cells(known);
        if peers.is_empty() {
            continue;
        }

        let mut action = Action::new_erase_cells(Strategy::Peer, peers, known);
        action.clue_cell_for_known(Verdict::Secondary, cell, known);

        if effects.add_action(action) && single {
            return Some(effects);
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}
