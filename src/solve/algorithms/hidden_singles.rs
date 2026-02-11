use super::*;

pub fn find_hidden_singles(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    for (cell, digits) in board.unsolved_iter() {
        for digit in digits {
            for house in cell.houses() {
                if board.house_candidate_cells(house, digit).len() == 1 {
                    let mut action = Action::new_set(Strategy::HiddenSingle, cell, digit);
                    action.clue_cells_for_digit(
                        Verdict::Related,
                        house.cells() - cell - board.solved(),
                        digit,
                    );

                    if effects.add_action(action) && single {
                        return Some(effects);
                    }
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
