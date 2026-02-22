use itertools::Itertools;

use super::*;

pub fn find_finned_swordfish(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    if !check_houses(board, single, Shape::Row, &mut effects) {
        check_houses(board, single, Shape::Column, &mut effects);
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

fn check_houses(board: &Board, single: bool, base_shape: Shape, effects: &mut Effects) -> bool {
    let cover_shape = match base_shape {
        Shape::Row => Shape::Column,
        Shape::Column => Shape::Row,
        Shape::Block => return false,
    };

    for digit in Digit::iter() {
        let base_houses: Vec<House> = base_shape.house_iter().collect();
        let cover_houses: Vec<House> = cover_shape.house_iter().collect();

        for base_combo in base_houses.iter().copied().combinations(3) {
            let base_cells: Vec<CellSet> = base_combo
                .iter()
                .map(|base| board.house_candidate_cells(*base, digit))
                .collect();

            if base_cells.iter().any(|cells| cells.len() < 2) {
                continue;
            }

            for cover_combo in cover_houses.iter().copied().combinations(3) {
                let cover_set: HouseSet = cover_combo.iter().copied().collect();
                if check_combo(
                    board,
                    digit,
                    base_shape,
                    &base_combo,
                    &base_cells,
                    cover_set,
                    single,
                    effects,
                ) {
                    return true;
                }
            }
        }
    }

    false
}

#[allow(clippy::too_many_arguments)]
fn check_combo(
    board: &Board,
    digit: Digit,
    base_shape: Shape,
    base_combo: &[House],
    base_cells: &[CellSet],
    cover_set: HouseSet,
    single: bool,
    effects: &mut Effects,
) -> bool {
    let cover_cells = cover_set.cells();
    let mut cover_used = HouseSet::empty(cover_set.shape());
    let mut fin_index = None;
    let mut fin_extra = CellSet::empty();

    for (index, cells) in base_cells.iter().enumerate() {
        let covered = *cells & cover_cells;
        let extra = *cells - covered;
        if extra.is_empty() {
            if covered.len() < 1 {
                return false;
            }
        } else {
            if fin_index.is_some() {
                return false;
            }
            if covered.is_empty() {
                return false;
            }
            fin_index = Some(index);
            fin_extra = extra;
        }
        cover_used |= base_combo[index].crossing_houses(covered);
    }

    let Some(fin_index) = fin_index else {
        return false;
    };

    if fin_extra.len() > 4 {
        return false;
    }

    if cover_used != cover_set {
        return false;
    }

    let fin_base = base_combo[fin_index];

    for fin_cover in cover_set.iter() {
        let fin_block = block_at(fin_base, fin_cover, base_shape);
        if !fin_extra.is_subset_of(fin_block.cells()) {
            continue;
        }

        let _ = cell_at(fin_base, fin_cover, base_shape);

        let cover_cells_all = cover_cells & board.candidate_cells(digit);
        let main_cells = base_cells.iter().copied().union_cells();
        let xwing_elims = cover_cells_all - main_cells;
        let erase = xwing_elims & fin_block.cells();
        if erase.is_empty() {
            continue;
        }

        let mut action = Action::new(Strategy::FinnedSwordfish);
        action.erase_cells(erase, digit);
        action.clue_cells_for_digit(Verdict::Primary, fin_extra, digit);
        base_cells.iter().for_each(|cells| {
            action.clue_cells_for_digit(Verdict::Secondary, *cells & cover_cells, digit);
        });

        if effects.add_action(action) && single {
            return true;
        }
    }

    false
}

fn cell_at(base: House, cover: House, base_shape: Shape) -> Cell {
    match base_shape {
        Shape::Row => Cell::from_row_column(base, cover),
        Shape::Column => Cell::from_row_column(cover, base),
        Shape::Block => unreachable!(),
    }
}

fn block_at(base: House, cover: House, base_shape: Shape) -> House {
    cell_at(base, cover, base_shape).block()
}

#[cfg(test)]
mod tests {
    use crate::io::{Parse, Parser};
    use crate::*;

    use super::*;

    fn assert_any_erase(actions: &[Action], cell: Cell, digit: Digit) {
        assert!(
            actions.iter().any(|action| action.erases(cell, digit)),
            "expected erase of {} from {}",
            digit,
            cell
        );
    }

    #[test]
    fn example_1() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B069i05040a089i2e020h7y7y92070b8e26010b019q92868i2y083a092e08020f052f2f040u0e3i0a0h7u0bae3a011o1o077u03088i050u088j9286ai33022e058i8j8m02ai2n2n0h070w0w081a01061a09",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_finned_swordfish(&board, false).expect("not found");
        assert_any_erase(got.actions(), cell!(G4), digit!(3));
    }

    #[test]
    fn sashimi_example_1() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B027ybi874nbb0f7n0gb707b60645048503857r0f057t077t080d7p0e080b7r0f7r7r070d047q0g080p057t7p067q010f9k0d9k7s050h5y12012w0960040f109i027q0413069v0883068abe2t4l5x9x7p03",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_finned_swordfish(&board, false).expect("not found");
        assert_any_erase(got.actions(), cell!(H1), digit!(3));
    }

    #[test]
    fn sashimi_example_2() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B04021i2b6q433b0905bq2u922t0d2r3d38604i2q01093o0304385w2u0612080i022q010d2q04020c012q09080f0a090h042q062s032c880a0706140408107s88120d2t6gdf383o9k0608862s2w9u2g0401",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_finned_swordfish(&board, false).expect("not found");
        assert_any_erase(got.actions(), cell!(B6), digit!(7));
    }

    #[test]
    fn double_sashimi_example() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B1w0g030d0810011u0i1g091f1413070d5e5e4q4q4b060i0z0g0c025e010g101u030i50040i56025u3f4z051f0c035m5609235h44074z07540i4m1y04484l4j565c56013q5e0o096a0a46055y020i060d5u",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_finned_swordfish(&board, false).expect("not found");
        assert!(got.action_count() > 0);
    }
}
