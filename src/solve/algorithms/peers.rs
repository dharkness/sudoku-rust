use super::*;

pub fn find_peers(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    for (cell, known) in board.known_iter() {
        let peers = cell.peers() & board.candidate_cells(known);
        let mut action = Action::new_erase_cells(Strategy::Peer, peers, known);
        action.clue_cell_for_known(Color::Blue, cell, known);

        effects.add_action(action);
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}
