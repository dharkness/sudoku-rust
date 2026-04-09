use itertools::Itertools;

use super::*;

pub fn find_finned_x_wings(board: &Board, single: bool) -> Option<Effects> {
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

fn check_houses(board: &Board, single: bool, shape: Shape, effects: &mut Effects) -> bool {
    for digit in Digit::iter() {
        for pair in shape.house_iter().combinations(2) {
            let base_a = pair[0];
            let base_b = pair[1];

            let cells_a = board.house_candidate_cells(base_a, digit);
            if cells_a.len() < 2 {
                continue;
            }
            let cells_b = board.house_candidate_cells(base_b, digit);
            if cells_b.len() < 2 {
                continue;
            }

            let crosses_a = base_a.crossing_houses(cells_a);
            let crosses_b = base_b.crossing_houses(cells_b);

            if check_pair(
                board, digit, shape, base_a, cells_a, crosses_a, base_b, cells_b, crosses_b,
                single, effects,
            ) {
                return true;
            }

            if check_pair(
                board, digit, shape, base_b, cells_b, crosses_b, base_a, cells_a, crosses_a,
                single, effects,
            ) {
                return true;
            }
        }
    }

    false
}

#[allow(clippy::too_many_arguments)]
fn check_pair(
    board: &Board,
    digit: Digit,
    base_shape: Shape,
    fin_base: House,
    fin_cells: CellSet,
    _fin_crosses: HouseSet,
    _clean_base: House,
    clean_cells: CellSet,
    clean_crosses: HouseSet,
    single: bool,
    effects: &mut Effects,
) -> bool {
    if clean_crosses.len() != 2 {
        return false;
    }

    let cover_set = clean_crosses;
    let cover_cells_fin = fin_cells & cover_set.cells();
    let extra_cells = fin_cells - cover_cells_fin;
    if extra_cells.is_empty() || extra_cells.len() > 2 {
        return false;
    }

    let cover_cells_clean = clean_cells & cover_set.cells();
    if cover_cells_clean.len() != 2 {
        return false;
    }

    let covers: Vec<House> = cover_set.iter().collect();
    if covers.len() != 2 {
        return false;
    }

    for fin_cover in covers.iter().copied() {
        let fin_block = block_at(fin_base, fin_cover, base_shape);
        if !extra_cells.is_subset_of(fin_block.cells()) {
            continue;
        }

        let other_cover = if fin_cover == covers[0] {
            covers[1]
        } else {
            covers[0]
        };

        let fin_corner = cell_at(fin_base, fin_cover, base_shape);
        let other_corner = cell_at(fin_base, other_cover, base_shape);

        let has_other_corner = fin_cells.has(other_corner);
        if !has_other_corner {
            continue;
        }

        let has_fin_corner = fin_cells.has(fin_corner);
        if !has_fin_corner && fin_cells.len() != extra_cells.len() + 1 {
            continue;
        }

        let cover_cells_all = cover_set.cells() & board.candidate_cells(digit);
        let main_cells = fin_cells | clean_cells;
        let xwing_elims = cover_cells_all - main_cells;
        let erase = xwing_elims & fin_block.cells();
        if erase.is_empty() {
            continue;
        }

        let mut action = Action::new(Strategy::FinnedXWing);
        action.erase_cells(erase, digit);
        action.clue_cells_for_digit(Verdict::Primary, extra_cells, digit);
        action.clue_cells_for_digit(
            Verdict::Secondary,
            cover_cells_clean | cover_cells_fin,
            digit,
        );

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
    use crate::puzzle::{ChangeResult, Changer, Options};
    use crate::solve::algorithms::{find_hidden_singles, find_naked_singles};
    use crate::*;

    use super::*;

    fn apply_singles(board: &Board) -> Board {
        let mut board = *board;
        let changer = Changer::new(Options::errors());

        loop {
            let mut progressed = false;
            for find in [find_naked_singles, find_hidden_singles] {
                let Some(mut effects) = find(&board, true) else {
                    continue;
                };
                let Some(action) = effects.pop_action() else {
                    continue;
                };
                match changer.apply(&board, &action) {
                    ChangeResult::Valid(after, _) => {
                        board = *after;
                        progressed = true;
                    }
                    ChangeResult::None => (),
                    ChangeResult::Invalid(..) => return board,
                }
                break;
            }

            if !progressed {
                return board;
            }
        }
    }

    #[test]
    fn figure_2_example() {
        let parser = Parse::packed_with_options(Options::errors());
        let (board, effects, failed) = parser.parse(
            "700008030036000000000050840205900000000745000000003604029010000000000910010500008",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_finned_x_wings(&board, false).expect("not found");
        assert_erase!(got, H9, 7);
    }

    #[test]
    fn sashimi_example() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B033e361m0a02050i0h7w180a7u088a0g060c8i081u071y8m0b0401078k1w8q2201030h1o1o980c0h07968q0a1o0a8q08021q8u8q0g0505010i0c1m081m020g1g03380a093e0h0e1m0h3604050b360a0c09",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_finned_x_wings(&board, false).expect("not found");
        assert_erase!(got, E6, 4);
        assert_erase!(got, F6, 4);
    }

    #[test]
    fn exemplar_example() {
        let parser = Parse::packed_with_options(Options::errors());
        let (board, effects, failed) = parser.parse(
            "900040000704080050080000100007600820620400000000000019000102000890700000000050003",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let board = apply_singles(&board);
        let got = find_finned_x_wings(&board, false).expect("not found");
        assert!(got.action_count() > 0);
    }
}
