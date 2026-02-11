use super::hidden_tuples::is_degenerate;
use super::*;

pub fn find_fireworks(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    for pivot in board.unsolved() {
        let row_cells = pivot.row().cells();
        let column_cells = pivot.column().cells();
        let block_cells = pivot.block().cells();
        let disjoint_cells = (row_cells | column_cells) - block_cells;
        let full_cells = disjoint_cells + pivot;
        let candidates =
            board.combined_candidates(row_cells) & board.combined_candidates(column_cells);
        for combos in candidates
            .iter()
            .filter_map(|digit| {
                let set = board.candidate_cells(digit);
                if set.has_any(row_cells) && set.has_any(column_cells) {
                    Some((digit, set))
                } else {
                    None
                }
            })
            .map(|(digit, set)| {
                (
                    digit,
                    set & block_cells,
                    set & disjoint_cells,
                    set & full_cells,
                )
            })
            .filter(|(_, block_set, disjoint_set, _)| {
                !block_set.is_empty() && disjoint_set.len() <= 2
            })
            .combinations(3)
        {
            let triple = combos.iter().map(|(digit, ..)| *digit).union_digits();
            if triple.len() != 3 {
                continue;
            }

            let wings = combos
                .iter()
                .map(|(_, _, disjoint_set, _)| *disjoint_set)
                .union_cells();
            if let Some((wing1, wing2)) = wings.as_pair() {
                if wing1.sees(wing2) {
                    continue;
                }

                let cells = wings + pivot;
                let all_digits = board.combined_candidates(cells);
                if !all_digits.has_all(triple) {
                    continue;
                }

                let full_sets = combos
                    .iter()
                    .map(|(_, _, _, full_set)| *full_set)
                    .collect_vec();
                if is_degenerate(&full_sets, 3, 2) {
                    continue;
                }

                let mut action = Action::new(Strategy::Fireworks);
                cells.iter().for_each(|cell| {
                    let digits = board.candidates(cell);
                    action.erase_digits(cell, digits - triple);
                    action.clue_cell_for_digits(Verdict::Secondary, cell, triple & digits);
                });

                if effects.add_action(action) && single {
                    return Some(effects);
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
    use crate::*;

    use super::*;

    #[test]
    fn found() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "0509gi2i2i8141i011l021g2055a5a81g80h50c08g7og17o032805o003219g1ghg0905410hk0096005m0118103c00511c84a4ag10h2128h0413g813g0503g828p0o03232050h41g8030h05g14848211181",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_fireworks(&board, true) {
            let mut action = Action::new(Strategy::Fireworks);
            action.erase_digits(cell!(C4), digits![4 5 6]);
            action.clue_cells_for_digit(Verdict::Secondary, cells![C4 F4], digit!(3));
            action.clue_cells_for_digit(Verdict::Secondary, cells![C4 F1 F4], digit!(7));
            action.clue_cells_for_digit(Verdict::Secondary, cells![F1 F4], digit!(8));

            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }

    #[test]
    fn candidate_must_not_be_solved_in_cross() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "4m811108k2060k21gk06g02230820h4108944k0960n0s03403gkpk080g04m0k222801162211280500h09g006461241g0810532090i2g8006080h11g12141065220420408801ggigig1140h032140148009",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        assert_eq!(None, find_fireworks(&board, true));
    }
}
