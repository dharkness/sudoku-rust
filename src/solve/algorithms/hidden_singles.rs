use super::*;

pub fn find_hidden_singles(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    for (cell, known) in board.known_iter() {
        for house in cell.houses() {
            if board.house_candidate_cells(house, known).size() == 1 {
                effects.add_set(Strategy::HiddenSingle, cell, known);
                break;
            }
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}
