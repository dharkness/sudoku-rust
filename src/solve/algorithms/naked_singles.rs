use super::*;

pub fn find_naked_singles(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    for (cell, digits) in board.cells_with_n_candidates_iter(1) {
        let digit = digits.as_single().unwrap();
        let mut action = Action::new_set(Strategy::NakedSingle, cell, digit);
        action.clue_cell_for_digits(Verdict::Related, cell, DigitSet::full() - digit);

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
