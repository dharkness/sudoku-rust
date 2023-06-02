use super::*;

pub fn find_intersection_removals(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    House::all_blocks().iter().for_each(|block| {
        block.rows().iter().for_each(|row| {
            check_intersection(board, *block, *row, &mut effects);
        });
        block.columns().iter().for_each(|column| {
            check_intersection(board, *block, *column, &mut effects);
        });
    });

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

fn check_intersection(board: &Board, block: House, other: House, effects: &mut Effects) {
    let segment = block.cells() & other.cells();
    let block_disjoint = block.cells() - segment;
    let other_disjoint = other.cells() - segment;
    let segment_candidates = board.all_candidates(segment);
    let block_disjoint_candidates = board.all_candidates(block_disjoint);
    let other_disjoint_candidates = board.all_candidates(other_disjoint);

    for known in Known::ALL {
        if segment_candidates[known] {
            let candidate_cells = board.candidate_cells(known);
            if block_disjoint_candidates[known] {
                if !other_disjoint_candidates[known] {
                    let erase = block_disjoint & candidate_cells;
                    if !erase.is_empty() {
                        let mut action = Action::new(Strategy::BoxLineReduction);
                        action.erase_cells(erase, known);
                        effects.add_action(action);
                    }
                }
            } else if other_disjoint_candidates[known] {
                let erase = other_disjoint & candidate_cells;
                if !erase.is_empty() {
                    let mut strategy = Strategy::PointingPair;
                    if segment_candidates.size() == 3 {
                        strategy = Strategy::PointingTriple;
                    }
                    let mut action = Action::new(strategy);
                    action.erase_cells(erase, known);
                    effects.add_action(action);
                }
            }
        }
    }
}
