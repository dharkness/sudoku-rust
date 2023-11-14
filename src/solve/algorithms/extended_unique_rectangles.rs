use super::*;

// https://www.sudokuwiki.org/extended_Unique_Rectangles
pub fn find_extended_unique_rectangles(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    let knowns = board.knowns();

    for (main, cross, blocks) in [
        (Shape::Row, Shape::Column, [0, 1, 2]),
        (Shape::Column, Shape::Row, [0, 3, 6]),
    ] {
        for t in 0..3 {
            let top = House::new(main, Coord::new(t));
            for m in 3..6 {
                let middle = House::new(main, Coord::new(m));
                for b in 6..9 {
                    let bottom = House::new(main, Coord::new(b));
                    let mains = HouseSet::empty(main) + top + middle + bottom;
                    for (shift, third) in blocks.iter().enumerate() {
                        for c in 0..3 {
                            let crosses = House::block(Coord::new(*third)).houses(cross)
                                - House::new(cross, Coord::new(3 * (shift as u8) + c));
                            let (left_cross, right_cross) = crosses.as_pair().unwrap();
                            let main_cells = top.cells() | middle.cells() | bottom.cells();
                            let left_cells = main_cells & left_cross.cells();
                            if knowns.has_any(left_cells) {
                                continue;
                            }
                            let right_cells = main_cells & right_cross.cells();
                            if knowns.has_any(right_cells) {
                                continue;
                            }
                            let left_candidates = board.all_candidates(left_cells);
                            let right_candidates = board.all_candidates(right_cells);
                            let common = left_candidates & right_candidates;
                            if common.len() < 3 {
                                continue;
                            }

                            if [
                                (left_candidates, right_candidates, right_cells),
                                (right_candidates, left_candidates, left_cells),
                            ]
                            .into_iter()
                            .any(|(subset, superset, cells)| {
                                subset.len() == 3
                                    && superset.len() > 3
                                    && superset.has_all(subset)
                                    && cells
                                        .iter()
                                        .map(|c| (c, board.candidates(c) - subset))
                                        .filter(|(_, ks)| !ks.is_empty())
                                        .count()
                                        == 1
                            }) {
                                // type 1
                                let mut action = Action::new(Strategy::ExtendedUniqueRectangle);
                                (left_cells | right_cells).iter().for_each(|c| {
                                    let candidates = board.candidates(c);
                                    if !(candidates - common).is_empty() {
                                        action.erase_knowns(c, candidates & common);
                                    } else {
                                        action.clue_cell_for_knowns(
                                            Verdict::Primary,
                                            c,
                                            candidates,
                                        );
                                    }
                                });

                                if effects.add_action(action) && single {
                                    return Some(effects);
                                }
                            } else {
                                // type 4
                                for out in mains {
                                    let in_cells = (mains - out).cells() & crosses.cells();
                                    let in_all = board.all_candidates(in_cells);
                                    if in_all.len() != 3 {
                                        continue;
                                    }
                                    let in_common = board.common_candidates(in_cells);
                                    if let Some(naked) = in_common.as_single() {
                                        let out_cells = out.cells() & crosses.cells();
                                        let origin = out_cells.first().unwrap();
                                        if let Some(keep) =
                                            board.all_candidates(in_cells).iter().find(|k| {
                                                [origin.row(), origin.column(), origin.block()]
                                                    .into_iter()
                                                    .any(|h| {
                                                        out_cells
                                                            == board.house_candidate_cells(h, *k)
                                                    })
                                            })
                                        {
                                            if let Some(erase) = (in_all - naked - keep).as_single()
                                            {
                                                let mut action =
                                                    Action::new(Strategy::ExtendedUniqueRectangle);
                                                in_cells.iter().for_each(|c| {
                                                    action.clue_cell_for_knowns(
                                                        Verdict::Primary,
                                                        c,
                                                        board.candidates(c),
                                                    );
                                                });
                                                out_cells.iter().for_each(|c| {
                                                    action.clue_cell_for_known(
                                                        Verdict::Secondary,
                                                        c,
                                                        keep,
                                                    );
                                                    if board.candidates(c).has(erase) {
                                                        action.erase(c, erase);
                                                    }
                                                });

                                                if effects.add_action(action) && single {
                                                    return Some(effects);
                                                }
                                            }
                                        }
                                    }
                                }
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
    fn type_1_vertical() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "g1224182050h09a011810h05092111g10341320912g182410ha48416d20h508ag121948c1ac20a050h214290g13472g1508a82420h8c1a121a0hg10581412141g12182118205090h0h058121410911g103",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_extended_unique_rectangles(&board, true) {
            let mut action = Action::new(Strategy::ExtendedUniqueRectangle);
            action.erase_knowns(cell!("C1"), knowns!("1 5"));
            action.clue_cells_for_known(Verdict::Primary, cells!("C3 E1 E3 G1 G3"), known!("1"));
            action.clue_cells_for_known(Verdict::Primary, cells!("E1 E3 G1 G3"), known!("3"));
            action.clue_cells_for_known(Verdict::Primary, cells!("C3 E1 G1 G3"), known!("5"));

            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }

    #[test]
    fn type_1_horizontal() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "68g14o812g0311054881110348g10521480h68054o482g11g103814e8ag1211148c80h4c0h8814g60648c8h0215c215cg4810h03l8kchc411c060e210h81gaga0a210h418105g8110c0h81110eg148214a",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_extended_unique_rectangles(&board, true) {
            let mut action = Action::new(Strategy::ExtendedUniqueRectangle);
            action.erase_knowns(cell!("D2"), knowns!("3 8"));
            action.clue_cells_for_known(Verdict::Primary, cells!("D6 D7 E2 E6 E7"), known!("3"));
            action.clue_cells_for_known(Verdict::Primary, cells!("D6 D7 E6 E7"), known!("7"));
            action.clue_cells_for_known(Verdict::Primary, cells!("D7 E2 E7"), known!("8"));

            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }

    #[test]
    fn type_4() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "g803419g21o0051g98050ho003p0099041211188218g0541o8gg0341g10o210i051a811803110588o8o041210h81210o410i11ga05g80h41p0989821o00305210590g190030h0941g8880305410h21h0p0",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_extended_unique_rectangles(&board, true) {
            let mut action = Action::new(Strategy::ExtendedUniqueRectangle);
            action.erase_cells(cells!("D7 F7"), known!("3"));
            action.clue_cells_for_known(Verdict::Primary, cells!("D5 F5"), known!("1"));
            action.clue_cells_for_known(Verdict::Secondary, cells!("D7 F7"), known!("1"));
            action.clue_cells_for_known(Verdict::Primary, cells!("D3 F3"), known!("3"));
            action.clue_cells_for_known(Verdict::Primary, cells!("D3 D5"), known!("4"));
            action.clue_cells_for_known(Verdict::Primary, cells!("F3 F5"), known!("4"));

            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }
}
