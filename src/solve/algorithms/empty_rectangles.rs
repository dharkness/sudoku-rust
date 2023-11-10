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
                                    action.clue_cell_for_known(Verdict::Secondary, start, known);
                                }
                                action.clue_cells_for_known(Verdict::Primary, cells, known);
                                action.clue_cell_for_known(Verdict::Secondary, pivot, known);

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

#[cfg(test)]
mod tests {
    use crate::io::{Parse, Parser};
    use crate::layout::cells::cell::cell;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::values::known::known;

    use super::*;

    #[test]
    fn test() {
        let parser = Parse::wiki().stop_on_error();
        let (board, ..) = parser.parse(
            "441181i402i4k4080h0g20g10884418411024c0c03o4100gs421g4p4o4410h09q403o030o6om0911a4o42go040p0og20o040031g0508g2g214a40ha409403020411403g108140g8188880g412411i402g4",
        );

        if let Some(got) = find_empty_rectangles(&board) {
            let mut action = Action::new(Strategy::EmptyRectangle);
            action.erase(cell!("J5"), known!("2"));
            action.clue_cells_for_known(Verdict::Primary, cells!("H7 J7 J9"), known!("2"));
            action.clue_cells_for_known(Verdict::Secondary, cells!("B5 B7"), known!("2"));

            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("No effects found");
        }
    }
}
