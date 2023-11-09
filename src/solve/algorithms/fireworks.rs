use super::hidden_tuples::is_degenerate;
use super::*;

pub fn find_fireworks(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    for pivot in Cell::iter() {
        let row_cells = pivot.row().cells();
        let column_cells = pivot.column().cells();
        let block_cells = pivot.block().cells();
        let disjoint_cells = (row_cells | column_cells) - block_cells;
        let full_cells = disjoint_cells + pivot;
        for candidates in Known::iter()
            .map(|known| {
                let set = board.candidate_cells(known);
                (
                    known,
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
            let triple = candidates.iter().map(|(known, ..)| *known).union_knowns();
            if triple.len() != 3 {
                continue;
            }

            let wings = candidates
                .iter()
                .map(|(_, _, disjoint_set, _)| *disjoint_set)
                .union_cells();
            if let Some((wing1, wing2)) = wings.as_pair() {
                if wing1.sees(wing2) {
                    continue;
                }

                let cells = wings + pivot;
                let all_knowns = board.all_candidates(cells);
                if !all_knowns.has_all(triple) {
                    continue;
                }

                let full_sets = candidates
                    .iter()
                    .map(|(_, _, _, full_set)| *full_set)
                    .collect_vec();
                if is_degenerate(&full_sets, 3, 2) {
                    continue;
                }

                let mut action = Action::new(Strategy::Fireworks);
                cells.iter().for_each(|cell| {
                    let knowns = board.candidates(cell);
                    action.erase_knowns(cell, knowns - triple);
                    action.clue_cell_for_knowns(Verdict::Secondary, cell, triple & knowns);
                });

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
    use crate::layout::values::known_set::knowns;

    use super::*;

    #[test]
    fn found() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "0509gi2i2i8141i011l021g2055a5a81g80h50c08g7og17o032805o003219g1ghg0905410hk0096005m0118103c00511c84a4ag10h2128h0413g813g0503g828p0o03232050h41g8030h05g14848211181",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_fireworks(&board) {
            let mut action = Action::new(Strategy::Fireworks);
            action.erase_knowns(cell!("C4"), knowns!("4 5 6"));
            action.clue_cells_for_known(Verdict::Secondary, cells!("C4 F4"), known!("3"));
            action.clue_cells_for_known(Verdict::Secondary, cells!("C4 F1 F4"), known!("7"));
            action.clue_cells_for_known(Verdict::Secondary, cells!("F1 F4"), known!("8"));

            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }

    #[test]
    fn not_found() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "4m8111kcka06gk21gk06i6i6j4o20h4108p44k09m4n4s0b403okpk8e0goem0k222o411u621h6o6l00h09o4o6s61241g281053209giii8e068e0h11g12141065232620c8884hggigig1140h032140948409",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        assert!(
            find_fireworks(&board).is_none(),
            "found unexpected fireworks"
        );
    }
}
