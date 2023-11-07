use super::*;

pub fn find_two_string_kites(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    for known in Known::iter() {
        let candidates = board.candidate_cells(known);
        if candidates.len() < 5 {
            continue;
        }

        for row in House::rows_iter() {
            let row_cells = board.house_candidate_cells(row, known);
            if row_cells.len() != 2 || row_cells.blocks().len() == 1 {
                continue;
            }

            for column in House::columns_iter() {
                let column_cells = board.house_candidate_cells(column, known);
                if column_cells.len() != 2
                    || !(row_cells & column_cells).is_empty()
                    || column_cells.blocks().len() == 1
                {
                    continue;
                }

                let (row_cell_left, row_cell_right) = row_cells.as_pair().unwrap();
                let (column_cell_high, column_cell_low) = column_cells.as_pair().unwrap();

                let pivots;
                let ends;
                if row_cell_left.block() == column_cell_high.block() {
                    pivots = row_cell_left + column_cell_high;
                    ends = row_cell_right + column_cell_low;
                } else if row_cell_left.block() == column_cell_low.block() {
                    pivots = row_cell_left + column_cell_low;
                    ends = row_cell_right + column_cell_high;
                } else if row_cell_right.block() == column_cell_high.block() {
                    pivots = row_cell_right + column_cell_high;
                    ends = row_cell_left + column_cell_low;
                } else if row_cell_right.block() == column_cell_low.block() {
                    pivots = row_cell_right + column_cell_low;
                    ends = row_cell_left + column_cell_high;
                } else {
                    continue;
                }

                let erase = ends.peers() & candidates;
                if erase.is_empty() {
                    continue;
                }

                let mut action = Action::new_erase_cells(Strategy::TwoStringKite, erase, known);
                action.clue_cells_for_known(Color::Blue, ends, known);
                action.clue_cells_for_known(Color::Purple, pivots, known);
                effects.add_action(action);
            }
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
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
        let (board, effects, failed) = parser.parse(
            "k88002lg041o214858480h0550211a1281g0k8112080k20a050g4a21g108020h054011810h0580091040g002200341112181g109050g10034g4g092081g0040408k0l0k2800g21528121kg05k21i12485a",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_two_string_kites(&board) {
            let mut action = Action::new(Strategy::TwoStringKite);
            action.erase(cell!("B4"), known!("5"));
            action.clue_cells_for_known(Color::Blue, cells!("B7 H4"), known!("5"));
            action.clue_cells_for_known(Color::Purple, cells!("H9 J7"), known!("5"));

            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }
}
