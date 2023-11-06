use super::*;

pub fn find_hidden_singles(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    for (cell, knowns) in board.unknown_iter() {
        for known in knowns {
            for house in cell.houses() {
                if board.house_candidate_cells(house, known).len() == 1 {
                    let mut action = Action::new_set(Strategy::HiddenSingle, cell, known);
                    action.add_known_cells(
                        Color::None,
                        known,
                        house.cells() - cell - board.knowns(),
                    );

                    effects.add_action(action);
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
