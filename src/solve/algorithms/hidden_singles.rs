use super::*;

pub fn find_hidden_singles(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    for (cell, knowns) in board.unknown_iter() {
        for known in knowns {
            for house in cell.houses() {
                if board.house_candidate_cells(house, known).size() == 1 {
                    effects.add_set(Strategy::HiddenSingle, cell, known);
                    break;
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
