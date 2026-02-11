use super::*;

pub fn find_peers(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    for (cell, digit) in board.solved_iter() {
        let peers = cell.peers() & board.candidate_cells(digit);
        if peers.is_empty() {
            continue;
        }

        let mut action = Action::new_erase_cells(Strategy::Peer, peers, digit);
        action.clue_cell_for_digit(Verdict::Secondary, cell, digit);

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
