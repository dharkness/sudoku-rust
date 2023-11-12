use super::*;

pub fn find_naked_singles(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    for (cell, knowns) in board.cell_candidates_with_n_candidates(1) {
        let known = knowns.as_single().unwrap();
        let mut action = Action::new_set(Strategy::NakedSingle, cell, known);
        action.clue_cell_for_knowns(Verdict::Related, cell, KnownSet::full() - known);

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
