use super::*;

pub fn find_naked_singles(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    for (cell, knowns) in board.cell_candidates_with_n_candidates(1) {
        effects.add_set(Strategy::NakedSingle, cell, knowns.as_single().unwrap());
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}
