use super::*;

// http://sudopedia.enjoysudoku.com/Avoidable_Rectangle.html
// http://forum.enjoysudoku.com/puzzle-with-uniqueness-type-3-t3073-30.html
pub fn find_avoidable_rectangles(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    let candidates = board.solved();

    // type 1
    Rectangle::iter()
        .map(|r| (r, r.cells - candidates))
        .filter_map(|(r, cs)| cs.as_single().map(|c| (r.with_origin(c), c)))
        .filter(|(r, _)| board.value(r.top_right) == board.value(r.bottom_left))
        .filter_map(|(r, c)| board.value(r.bottom_right).known().map(|k| (r, c, k)))
        .filter(|(_, c, k)| board.candidates(*c).has(*k))
        .for_each(|(r, c, k)| {
            let mut action = Action::new_erase(Strategy::AvoidableRectangle, c, k);
            board
                .knowns_iter(r.cells & candidates)
                .for_each(|(cell, known)| {
                    action.clue_cell_for_known(Verdict::Secondary, cell, known)
                });
            effects.add_action(action);
        });

    for rect in Rectangle::iter() {
        if rect.cells.has_any(board.givens()) {
            continue;
        }

        let unsolved = rect.cells - board.knowns();
        if let Some((c1, c2)) = unsolved.as_pair() {
            let houses = c1.common_houses(c2);
            if houses.is_empty() {
                continue;
            }

            let mut action = Action::new(Strategy::AvoidableRectangle);
            if let Some((c3, c4)) = (rect.cells - unsolved).as_pair() {
                let ks1 = board.candidates(c1);
                let ks2 = board.candidates(c2);
                let k3 = board.value(c3).known().unwrap();
                let k4 = board.value(c4).known().unwrap();
                if !(ks1.has(k4) && ks2.has(k3)) {
                    continue;
                }
                action.clue_cell_for_known(Verdict::Tertiary, c3, k3);
                action.clue_cell_for_known(Verdict::Tertiary, c4, k4);
            } else {
                continue;
            }

            let mut pseudo = board.pseudo_cell(unsolved);
            let solved = board.all_knowns(rect.cells - unsolved);
            pseudo.knowns -= solved;

            unsolved.iter().for_each(|c| {
                let cs = board.candidates(c);
                action.clue_cell_for_knowns(Verdict::Tertiary, c, cs & solved);
                action.clue_cell_for_knowns(Verdict::Secondary, c, cs - solved);
            });
            if let Some(k) = pseudo.knowns.as_single() {
                // type 2 - naked single
                for house in houses {
                    action.erase_cells(board.house_candidate_cells(house, k) - unsolved, k);
                }

                effects.add_action(action);
            } else {
                // type 3 - naked tuple
                for house in houses {
                    let peers = house.cells() - rect.cells;
                    for size in 2..=4 {
                        peers
                            .iter()
                            .map(|cell| (cell, board.candidates(cell)))
                            .filter(|(_, knowns)| !knowns.has_any(solved))
                            .filter(|(_, knowns)| (2..=size).contains(&knowns.len()))
                            .combinations(size - 1)
                            .for_each(|peer_knowns| {
                                let known_sets: Vec<KnownSet> = peer_knowns
                                    .iter()
                                    .map(|(_, ks)| *ks)
                                    .chain([pseudo.knowns])
                                    .collect();
                                let knowns = known_sets.iter().copied().union_knowns();
                                if knowns.len() != size
                                    || naked_tuples::is_degenerate(&known_sets, size, 2)
                                    || naked_tuples::is_degenerate(&known_sets, size, 3)
                                {
                                    return;
                                }

                                let tuple_cells = peer_knowns.iter().map(|(c, _)| *c).union_cells();
                                let erase_cells = peers - tuple_cells;

                                tuple_cells.iter().for_each(|c| {
                                    action.clue_cell_for_knowns(
                                        Verdict::Secondary,
                                        c,
                                        knowns & board.candidates(c),
                                    );
                                });
                                knowns.iter().for_each(|k| {
                                    action.erase_cells(erase_cells & board.candidate_cells(k), k)
                                });
                            });
                    }
                }

                effects.add_action(action);

                // degenerates should create actions
                // normally, when looking for a naked triple, finding two cells
                // that collectively can only be two of the knowns
                // would be found by looking for naked pairs,
                // but since a pseudo cell is involved, it wouldn't be found.
                // thus, this should report them, maybe combining it with the triple
                // by removing the pair from the pseudo cell as well.
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
            "g0110g08a4a402a04040210211o00h0588g8040881i041031g3ghg0h0250k0h409211481300478cgbga0g01o0281g138033k34411s1s098g30ag02g09g4404308gj005bg4108033g024105ag09b09gg13g",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_avoidable_rectangles(&board) {
            let mut action = Action::new(Strategy::AvoidableRectangle);
            action.erase(cell!("B9"), known!("9"));
            action.clue_cells_for_known(Verdict::Secondary, cells!("A1"), known!("9"));
            action.clue_cells_for_known(Verdict::Secondary, cells!("A9 B1"), known!("7"));

            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }

    #[test]
    fn type_2() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "g0110g08a4a402a04040210211o00h0588g8040881i041031g3ghg0h0250k0h409211481300478cgbga0g01o0281g138033k34411s1s098g30ag02g09g4404308gj005bg4108033g024105ag09b09gg13g",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_avoidable_rectangles(&board) {
            let mut action = Action::new(Strategy::AvoidableRectangle);
            action.erase(cell!("B9"), known!("9"));
            action.clue_cells_for_known(Verdict::Secondary, cells!("A1"), known!("9"));
            action.clue_cells_for_known(Verdict::Secondary, cells!("A9 B1"), known!("7"));

            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }

    #[test]
    fn type_3() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "l080l80520035o1g50020h6008801060g104300438g0400g380280gg08gg4111800421020520030gg008508050815050210204g0080g1gg1800209401g04207g507g8004g0031g09080204100h208140g0",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_avoidable_rectangles(&board) {
            let mut action = Action::new(Strategy::AvoidableRectangle);
            action.erase_knowns(cell!("H1"), knowns!("4 5"));
            action.clue_cells_for_knowns(Verdict::Secondary, cells!("A1"), knowns!("5 9"));
            action.clue_cells_for_known(Verdict::Secondary, cells!("C1"), known!("5"));
            action.clue_cells_for_knowns(Verdict::Secondary, cells!("D1"), knowns!("4 9"));
            action.clue_cells_for_knowns(Verdict::Secondary, cells!("G1"), knowns!("4 5"));
            action.clue_cells_for_known(Verdict::Tertiary, cells!("A1 C5"), known!("7"));
            action.clue_cells_for_known(Verdict::Tertiary, cells!("A5 C1"), known!("6"));

            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }
}
