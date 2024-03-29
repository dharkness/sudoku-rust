use super::*;

/// Looks for a candidate remaining in two or three cells in a block
/// that all lie in one row or column.
///
/// - When the candidate is removed from the column/row disjoint,
///   the candidate cells in the block are called a Pointing Pair/Triple.
/// - When the candidate is removed from the block disjoint, it is called
///   a Box Line Reduction.
///
/// # Example
///
/// This shows one Pointing Pair and one Box Line Reduction.
///
/// ```text
///   123456789
/// A ·········
/// B ···888···  ←-- cells in box 5 (D5 E5) point to cell (B5)
/// C ·········
/// D ····8····
/// E ····8····
/// F ·········
/// G ·88······  ←-- cells in line (G4...G9) being empty
/// H ······88·
/// J 8·8··8···  ←-- ... reduce box 7, removing 8 from cells (J1 J3)
/// ```
pub fn find_intersection_removals(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    for block in House::blocks_iter() {
        if check_intersection(board, single, block, block.rows(), &mut effects)
            || check_intersection(board, single, block, block.columns(), &mut effects)
        {
            break;
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

fn check_intersection(
    board: &Board,
    single: bool,
    block: House,
    houses: HouseSet,
    effects: &mut Effects,
) -> bool {
    for known in Known::iter() {
        for house in houses.iter() {
            let block_cells = block.cells();
            let intersection_cells = block_cells & house.cells();
            let box_cells = block_cells - intersection_cells;
            let box_candidates = board.all_candidates(box_cells);
            let line_cells = house.cells() - intersection_cells;
            let line_candidates = board.all_candidates(line_cells);

            let candidate_cells = board.candidate_cells(known);
            let intersection_candidate_cells = intersection_cells & candidate_cells;
            let intersection_candidate_cells_count = intersection_candidate_cells.len();
            if intersection_candidate_cells_count < 2 {
                // ignore hidden single
                continue;
            }

            if box_candidates[known] {
                if !line_candidates[known] {
                    let erase = box_cells & candidate_cells;
                    if !erase.is_empty() {
                        let mut action = Action::new(Strategy::BoxLineReduction);
                        action.erase_cells(erase, known);
                        action.clue_cells_for_known(
                            Verdict::Secondary,
                            intersection_candidate_cells,
                            known,
                        );
                        action.clue_cells_for_known(
                            Verdict::Related,
                            line_cells - board.knowns(),
                            known,
                        );

                        if effects.add_action(action) && single {
                            return true;
                        }
                    }
                }
            } else if line_candidates[known] {
                let erase = line_cells & candidate_cells;
                if !erase.is_empty() {
                    let mut strategy = Strategy::PointingPair;
                    if intersection_candidate_cells_count == 3 {
                        strategy = Strategy::PointingTriple;
                    }
                    let mut action = Action::new(strategy);
                    action.erase_cells(erase, known);
                    action.clue_cells_for_known(
                        Verdict::Secondary,
                        intersection_candidate_cells,
                        known,
                    );
                    action.clue_cells_for_known(
                        Verdict::Related,
                        block_cells - intersection_cells - board.knowns(),
                        known,
                    );

                    if effects.add_action(action) && single {
                        return true;
                    }
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use crate::io::{Parse, Parser};
    use crate::layout::cells::cell_set::cells;
    use crate::layout::values::known::known;

    use super::*;

    #[test]
    fn intersection_removals() {
        let board = Parse::packed_with_options(Options::errors()).parse_simple(
            "
                7..1....9
                .2.3..7..
                4.9......
                .6.8..2..
                .........
                .7...1.5.
                .....49..
                .46..5..2
                .1...68..
            ",
        );

        let found = find_intersection_removals(&board, false).unwrap_or(Effects::new());
        assert_eq!(cells!("B8 B9"), found.erases_from_cells(known!("1")));
        assert_eq!(cells!(""), found.erases_from_cells(known!("2")));
        assert_eq!(cells!("D5 E5 F5"), found.erases_from_cells(known!("3")));
        assert_eq!(cells!("D5 E5 F5"), found.erases_from_cells(known!("4")));
        assert_eq!(cells!("B9 C9"), found.erases_from_cells(known!("5")));
        assert_eq!(cells!(""), found.erases_from_cells(known!("6")));
        assert_eq!(cells!(""), found.erases_from_cells(known!("7")));
        assert_eq!(cells!("A5 B5 C5"), found.erases_from_cells(known!("8")));
        assert_eq!(cells!("D1 E1 F1"), found.erases_from_cells(known!("9")));
    }
}
