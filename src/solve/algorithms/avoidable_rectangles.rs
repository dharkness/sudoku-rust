use super::*;

// http://sudopedia.enjoysudoku.com/Avoidable_Rectangle.html
// http://forum.enjoysudoku.com/puzzle-with-uniqueness-type-3-t3073-30.html
pub fn find_avoidable_rectangles(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    let givens = board.givens();
    let solved = board.placed();

    // type 1
    for (r, c, d) in Rectangle::iter()
        .filter(|r| !r.cells.has_any(givens))
        .map(|r| (r, r.cells - solved))
        .filter_map(|(r, cs)| cs.as_single().map(|c| (r.with_origin(c), c)))
        .filter(|(r, _)| board.value(r.top_right) == board.value(r.bottom_left))
        .filter_map(|(r, c)| board.value(r.bottom_right).digit().map(|d| (r, c, d)))
        .filter(|(_, c, d)| board.candidates(*c).has(*d))
    {
        let mut action = Action::new_erase(Strategy::AvoidableRectangle, c, d);
        board
            .solved_subset_iter(r.cells & solved)
            .for_each(|(cell, digit)| action.clue_cell_for_digit(Verdict::Primary, cell, digit));

        if effects.add_action(action) && single {
            return Some(effects);
        }
    }

    for rect in Rectangle::iter() {
        if rect.cells.has_any(givens) {
            continue;
        }

        let unsolved = rect.cells - board.solved();
        if let Some((c1, c2)) = unsolved.as_pair() {
            let houses = c1.common_houses(c2);
            if houses.is_empty() {
                continue;
            }

            let mut action = Action::new(Strategy::AvoidableRectangle);
            if let Some((c3, c4)) = (rect.cells - unsolved).as_pair() {
                let ds1 = board.candidates(c1);
                let ds2 = board.candidates(c2);
                let d3 = board.value(c3).digit().unwrap();
                let d4 = board.value(c4).digit().unwrap();
                if !(ds1.has(d4) && ds2.has(d3)) {
                    continue;
                }
                action.clue_cell_for_digit(Verdict::Primary, c3, d3);
                action.clue_cell_for_digit(Verdict::Primary, c4, d4);
            } else {
                continue;
            }

            let mut pseudo = board.pseudo_cell(unsolved);
            let solved = board.solved_subset_digits(rect.cells - unsolved);
            pseudo.digits -= solved;

            unsolved.iter().for_each(|c| {
                let cs = board.candidates(c);
                action.clue_cell_for_digits(Verdict::Primary, c, cs & solved);
                action.clue_cell_for_digits(Verdict::Secondary, c, cs - solved);
            });
            if let Some(d) = pseudo.digits.as_single() {
                // type 2 - naked single
                for house in houses {
                    action.erase_cells(board.house_candidate_cells(house, d) - unsolved, d);
                }

                if effects.add_action(action) && single {
                    return Some(effects);
                }
            } else {
                // type 3 - naked tuple
                for house in houses {
                    let peers = house.cells() - rect.cells;
                    for size in 2..=4 {
                        peers
                            .iter()
                            .map(|cell| (cell, board.candidates(cell)))
                            .filter(|(_, digits)| !digits.has_any(solved))
                            .filter(|(_, digits)| (2..=size).contains(&digits.len()))
                            .combinations(size - 1)
                            .for_each(|peer_digits| {
                                let digit_sets: Vec<DigitSet> = peer_digits
                                    .iter()
                                    .map(|(_, ds)| *ds)
                                    .chain([pseudo.digits])
                                    .collect();
                                let digits = digit_sets.iter().copied().union_digits();
                                if digits.len() != size
                                    || naked_tuples::is_degenerate(&digit_sets, size, 2)
                                    || naked_tuples::is_degenerate(&digit_sets, size, 3)
                                {
                                    return;
                                }

                                let tuple_cells = peer_digits.iter().map(|(c, _)| *c).union_cells();
                                let erase_cells = peers - tuple_cells;

                                tuple_cells.iter().for_each(|c| {
                                    action.clue_cell_for_digits(
                                        Verdict::Secondary,
                                        c,
                                        digits & board.candidates(c),
                                    );
                                });
                                digits.iter().for_each(|d| {
                                    action.erase_cells(erase_cells & board.candidate_cells(d), d)
                                });
                            });
                    }
                }

                if effects.add_action(action) && single {
                    return Some(effects);
                }

                // degenerates should create actions
                // normally, when looking for a naked triple, finding two cells
                // that collectively can only be two of the digits
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
    use crate::*;

    use super::*;

    #[test]
    fn type_1() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "g0110g08a4a402a04040210211o00h0588g8040881i041031g3ghg0h0250k0h409211481300478cgbga0g01o0281g138033k34411s1s098g30ag02g09g4404308gj005bg4108033g024105ag09b09gg13g",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_avoidable_rectangles(&board, true) {
            let mut action = Action::new(Strategy::AvoidableRectangle);
            action.erase(cell!(B9), digit!(9));
            action.clue_cells_for_digit(Verdict::Primary, cells![A1], digit!(9));
            action.clue_cells_for_digit(Verdict::Primary, cells![A9 B1], digit!(7));

            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }

    #[test]
    fn type_2() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "21hg0540gg03800oh8kg09l0048120gg0ih2oggio218gg180521414426620i090k11g18008g6g28111412g0m2610810hg221g40840060341091g041gi080i0okikq0g841o802110co4h4p02102o8410c0h",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_avoidable_rectangles(&board, true) {
            let mut action = Action::new(Strategy::AvoidableRectangle);
            action.erase(cell!(E9), digit!(2));
            action.clue_cells_for_digit(Verdict::Primary, cells![F9 H7], digit!(1));
            action.clue_cells_for_digit(Verdict::Primary, cells![F7 H9], digit!(3));
            action.clue_cells_for_digit(Verdict::Secondary, cells![F9 H9], digit!(2));

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

        if let Some(got) = find_avoidable_rectangles(&board, true) {
            let mut action = Action::new(Strategy::AvoidableRectangle);
            action.erase_digits(cell!(H1), digits![4 5]);
            action.clue_cells_for_digits(
                Verdict::Secondary,
                CellSet::from(cell!(A1)),
                digits![5 9],
            );
            action.clue_cells_for_digit(Verdict::Secondary, cells![C1], digit!(5));
            action.clue_cells_for_digits(
                Verdict::Secondary,
                CellSet::from(cell!(D1)),
                digits![4 9],
            );
            action.clue_cells_for_digits(
                Verdict::Secondary,
                CellSet::from(cell!(G1)),
                digits![4 5],
            );
            action.clue_cells_for_digit(Verdict::Primary, cells![A1 C5], digit!(7));
            action.clue_cells_for_digit(Verdict::Primary, cells![A5 C1], digit!(6));

            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }
}
