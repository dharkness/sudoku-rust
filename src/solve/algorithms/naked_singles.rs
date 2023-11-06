use super::*;

pub fn find_naked_singles(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    for (cell, knowns) in board.cell_candidates_with_n_candidates(1) {
        let known = knowns.as_single().unwrap();
        let mut action = Action::new_set(Strategy::NakedSingle, cell, known);
        action.add_cell_knowns(Color::None, cell, KnownSet::full() - known);

        effects.add_action(action);
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}
