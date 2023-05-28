use crate::effects::{Effects, Strategy};
use crate::layout::{Board, Cell, House, Known};

pub fn find_intersection_removals(board: &Board) -> Effects {
    let mut effects = Effects::new();

    for block in House::all_blocks() {
        for row in block.rows() {
            check_intersection(board, *block, *row, &mut effects)
        }
        for column in block.columns() {
            check_intersection(board, *block, *column, &mut effects)
        }
    }

    effects
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
            if block_disjoint_candidates[known] {
                if !other_disjoint_candidates[known] {
                    for cell in block_disjoint.iter() {
                        if board.is_candidate(cell, known) {
                            effects.add_erase(Strategy::BoxLineReduction, cell, known);
                        }
                    }
                }
            } else {
                if other_disjoint_candidates[known] {
                    for cell in other_disjoint.iter() {
                        if board.is_candidate(cell, known) {
                            effects.add_erase(
                                if segment_candidates.size() == 3 {
                                    Strategy::PointingTriple
                                } else {
                                    Strategy::PointingPair
                                },
                                cell,
                                known,
                            );
                        }
                    }
                }
            }
        }
    }
}
