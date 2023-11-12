use super::*;

// https://www.sudokuwiki.org/Hidden_Unique_Rectangles
pub fn find_hidden_unique_rectangles(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    let knowns = board.knowns();
    let bi_values = board.cells_with_n_candidates(2);
    let mut possible_type_2s = Vec::new();

    'rect: for mut rect in Rectangle::iter() {
        if rect.cells.has_any(knowns) {
            continue;
        }

        let rect_bi_values = rect.cells & bi_values;
        if let Some(origin) = rect_bi_values.first() {
            let pair = board.candidates(origin);
            if !board.common_candidates(rect.cells).has_all(pair) {
                continue;
            }

            match rect_bi_values.len() {
                1 => {
                    // type 1
                    rect = rect.with_origin(origin);
                    for known in pair {
                        let opposite = rect.bottom_right;
                        if let Some(row) = opposite.common_row_or_column(rect.bottom_left) {
                            if let Some(column) = opposite.common_row_or_column(rect.top_right) {
                                if board.house_candidate_cells(row, known).len() == 2
                                    && board.house_candidate_cells(column, known).len() == 2
                                {
                                    let mut action = Action::new_erase_knowns(
                                        Strategy::HiddenUniqueRectangle,
                                        opposite,
                                        pair - known,
                                    );
                                    action.clue_cells_for_knowns(
                                        Verdict::Primary,
                                        rect.cells - opposite,
                                        pair,
                                    );
                                    action.clue_cell_for_known(Verdict::Primary, opposite, known);

                                    if effects.add_action(action) && single {
                                        return Some(effects);
                                    }

                                    continue 'rect;
                                }
                            }
                        }
                    }
                }
                2 => {
                    possible_type_2s.push((rect, rect_bi_values, pair));
                    continue;
                }
                _ => continue,
            }
        }
    }

    // type 2/2b
    for (rect, floor, pair) in possible_type_2s {
        if floor.share_row_or_column() {
            let roof = rect.cells - floor;
            let (roof1, roof2) = roof.as_pair().unwrap();
            let (floor1, floor2) = floor.as_pair().unwrap();

            for known in pair {
                for (wall1, wall2, erase) in [(floor1, roof1, roof2), (floor2, roof2, roof1)] {
                    if let Some(house) = wall1.common_row_or_column(wall2) {
                        if board.house_candidate_cells(house, known).len() == 2 {
                            let mut action = Action::new_erase_knowns(
                                Strategy::HiddenUniqueRectangle,
                                erase,
                                pair - known,
                            );
                            action.clue_cells_for_knowns(
                                Verdict::Primary,
                                rect.cells - erase,
                                pair,
                            );
                            action.clue_cell_for_known(Verdict::Primary, erase, known);

                            if effects.add_action(action) && single {
                                return Some(effects);
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

#[cfg(test)]
mod tests {
    use crate::io::{Parse, Parser};
    use crate::layout::cells::cell::cell;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::values::known::known;
    use crate::layout::values::known_set::knowns;

    use super::*;

    #[test]
    fn type_1() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "030kg11141ak0924akb0413009g10ka0030ka00k09ag03ak11g141308132410h09i234j40hg1052211224181094109320581g1220h30090341o005900h30r005218gg2091io041p0g1118g8g2141050903",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_hidden_unique_rectangles(&board, true) {
            let mut action = Action::new(Strategy::HiddenUniqueRectangle);
            action.erase(cell!("D3"), known!("6"));
            action.clue_cells_for_knowns(Verdict::Primary, cells!("D7 F3 F7"), knowns!("1 6"));
            action.clue_cells_for_known(Verdict::Primary, cells!("D3"), known!("1"));

            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }

    #[test]
    fn type_2() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "114g4g05g103810921a009a00h4111g403g40606g10981210h1141q411e0030h0964q0o40h24a84130g13ca003i2626a813005l80hh009ig3g30058g0341pg22813ig1094130051g41ig0530038gj0q009",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_hidden_unique_rectangles(&board, true) {
            let mut action = Action::new(Strategy::HiddenUniqueRectangle);
            action.erase(cell!("D3"), known!("6"));
            action.clue_cells_for_knowns(Verdict::Primary, cells!("B1 B3 D1"), knowns!("6 8"));
            action.clue_cells_for_known(Verdict::Primary, cells!("D3"), known!("8"));

            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }

    #[test]
    fn type_2b() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "4i05kg1181igm009k20911k0g606i6m0810h0i8121410ogoh005h2440h810cg14c031121114246214481g80hg8g121090i110i0541814kg14k0s211181034821425i814q0q5og10581095mgm4mkm5g2150",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_hidden_unique_rectangles(&board, true) {
            let mut action = Action::new(Strategy::HiddenUniqueRectangle);
            action.erase(cell!("H3"), known!("7"));
            action.clue_cells_for_knowns(Verdict::Primary, cells!("E2 E3 H2"), knowns!("1 7"));
            action.clue_cells_for_known(Verdict::Primary, cells!("H3"), known!("1"));

            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }
}
