use std::collections::HashMap;

use super::*;

pub fn find_wxyz_wings(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    let wing_candidates = (2..=4).fold(CellSet::empty(), |set, n| {
        set | board.cells_with_n_candidates(n)
    });

    for wing in wing_candidates.into_iter().combinations(4) {
        let wing = wing.into_iter().union_cells();
        if wing.rows().len() == 1 || wing.columns().len() == 1 || wing.blocks().len() == 1 {
            continue;
        }

        let wing_knowns = wing
            .iter()
            .fold(KnownSet::empty(), |set, cell| set | board.candidates(cell));
        if wing_knowns.len() != 4 {
            continue;
        }

        let mut restricted: HashMap<Known, CellSet> = HashMap::new();
        let mut non_restricted: HashMap<Known, CellSet> = HashMap::new();
        for known in wing_knowns {
            let candidates = wing & board.candidate_cells(known);
            let is_restricted = candidates
                .iter()
                .combinations(2)
                .all(|combo| combo[0].sees(combo[1]));
            if is_restricted {
                restricted.insert(known, candidates);
            } else {
                non_restricted.insert(known, candidates);
            }
        }
        if non_restricted.len() != 1 {
            continue;
        }

        let (candidate, cells) = non_restricted.into_iter().next().unwrap();
        let erase = cells
            .iter()
            .fold(board.candidate_cells(candidate), |set, cell| {
                set & cell.peers()
            });
        if erase.is_empty() {
            continue;
        }

        let mut action = Action::new_erase_cells(Strategy::WXYZWing, erase, candidate);
        action.clue_cells_for_known(Verdict::Secondary, cells, candidate);
        for (known, cells) in restricted {
            action.clue_cells_for_known(Verdict::Primary, cells, known);
        }

        effects.add_action(action);
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
    fn test_all_in_pivot() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "q2i2o2p2050h410992110ho20941o0052182050941b212b0h2o20h41j6h6h009h4810h32i0j4090h8103h0k470g2810hl021l4h20609090686e20ie02g11g18i4190b2g1092g8205oih221051i9009c2c2",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_wxyz_wings(&board) {
            assert_eq!(1, got.actions().len());

            let mut action = Action::new(Strategy::WXYZWing);
            action.erase(cell!("D2"), known!("9"));
            action.clue_cells_for_known(Verdict::Secondary, cells!("D3 D4 D6 F1"), known!("9"));
            action.clue_cells_for_known(Verdict::Primary, cells!("D3 F1"), known!("1"));
            action.clue_cells_for_known(Verdict::Primary, cells!("D3 D6"), known!("2"));
            action.clue_cells_for_known(Verdict::Primary, cells!("D3 D4 D6"), known!("5"));

            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }

    #[test]
    fn test_not_all_in_pivot() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "810h053030094103g160m00903s00hb0942411m00344s4o4a0090hi009812403i40h4111gg051g09h0410321810330413g9gb005g1090990215k5k14g19g034gd01gg10903b094240503g1813g30091g41",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_wxyz_wings(&board) {
            assert_eq!(1, got.actions().len());

            let mut action = Action::new(Strategy::WXYZWing);
            action.erase_cells(cells!("F6 G5 J5"), known!("5"));
            action.clue_cells_for_known(Verdict::Secondary, cells!("E5 G6 J6"), known!("5"));
            action.clue_cells_for_known(Verdict::Primary, cells!("D6 J6"), known!("6"));
            action.clue_cells_for_known(Verdict::Primary, cells!("D6 G6"), known!("2"));
            action.clue_cells_for_known(Verdict::Primary, cells!("D6 E5"), known!("9"));

            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }
}
