use super::*;

pub fn find_peers(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    for (cell, known) in board.known_iter() {
        let peers = cell.peers() & board.candidate_cells(known);
        effects.add_erase_cells(Strategy::Peer, peers, known);
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}
