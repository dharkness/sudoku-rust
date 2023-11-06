use super::*;

pub fn find_empty_rectangles(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    for known in Known::iter() {
        for block in House::blocks_iter() {
            if let Some((cells, row, column)) = fit_row_column(board, block, known) {
                let mut erased = CellSet::empty();
                // top, left, right and bottom are general terms about the formed rectangle:
                // bottom may be above top, and top and bottom will be columns in the second loop
                for (top, left) in [(row, column), (column, row)] {
                    // look for start and pivot cells as only candidates in right house
                    // to remove a candidate at the left and bottom house intersection
                    let candidates = board.house_candidate_cells(left, known) - cells;

                    for start in (board.house_candidate_cells(top, known) - cells).iter() {
                        if erased.has(start) {
                            continue;
                        }

                        let right = start.house(left.shape());
                        if let Some(pivot) =
                            (board.house_candidate_cells(right, known) - start).as_single()
                        {
                            if start.block() == pivot.block() {
                                // can't remove a cell from the starting block
                                continue;
                            }

                            let bottom = pivot.house(top.shape());
                            let ends = board.house_candidate_cells(bottom, known) - pivot;

                            if let Some(end) = (ends & candidates).as_single() {
                                erased += end;

                                let mut action =
                                    Action::new_erase(Strategy::EmptyRectangle, end, known);
                                if ends.len() == 1 {
                                    action.erase(start, known);
                                } else {
                                    action.clue_cell_for_known(Color::Blue, start, known);
                                }
                                action.clue_cell_for_known(Color::Blue, pivot, known);
                                action.clue_cells_for_known(Color::Red, cells, known);

                                effects.add_action(action);
                            }
                        }
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

fn fit_row_column(board: &Board, block: House, known: Known) -> Option<(CellSet, House, House)> {
    let cells = board.house_candidate_cells(block, known);
    if cells.len() < 3 {
        // possible degenerate singles chain if two and not a candidate if only one
        return None;
    }

    for row in block.rows().iter() {
        for column in block.columns().iter() {
            if cells.is_subset_of(row.cells() | column.cells()) {
                return Some((cells, row, column));
            }
        }
    }

    None
}
