use super::*;

/// Looks for a candidate remaining in two or three cells in a block
/// that all lie in one row or column.
///
/// - When the candidate is removed from the column/row disjoint,
///   the candidate cells in the block are called a Pointing Pair/Triple.
/// - When the candidate is removed from the block disjoint, it is called
///   a Box Line Reduction.

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
pub fn find_intersection_removals(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    House::blocks_iter().for_each(|block| {
        check_intersection(board, block, block.rows(), &mut effects);
        check_intersection(board, block, block.columns(), &mut effects);
    });

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

fn check_intersection(board: &Board, block: House, houses: HouseSet, effects: &mut Effects) {
    for known in Known::iter() {
        houses.iter().for_each(|house| {
            let segment = block.cells() & house.cells();
            let block_disjoint = block.cells() - segment;
            let other_disjoint = house.cells() - segment;
            let block_disjoint_candidates = board.all_candidates(block_disjoint);
            let other_disjoint_candidates = board.all_candidates(other_disjoint);

            let candidate_cells = board.candidate_cells(known);
            let segment_candidate_cells_count = (segment & candidate_cells).size();
            if segment_candidate_cells_count < 2 {
                return;
            }

            if block_disjoint_candidates[known] {
                if !other_disjoint_candidates[known] {
                    let erase = block_disjoint & candidate_cells;
                    if !erase.is_empty() {
                        let mut action = Action::new(Strategy::BoxLineReduction);
                        action.erase_cells(erase, known);
                        effects.add_action(action);
                    }
                }
            } else if other_disjoint_candidates[known] {
                let erase = other_disjoint & candidate_cells;
                if !erase.is_empty() {
                    let mut strategy = Strategy::PointingPair;
                    if segment_candidate_cells_count == 3 {
                        strategy = Strategy::PointingTriple;
                    }
                    let mut action = Action::new(strategy);
                    action.erase_cells(erase, known);
                    effects.add_action(action);
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::Parse;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::values::known::known;

    #[test]
    fn intersection_removals() {
        let board = Parse::packed_with_options(Options::errors_and_peers()).parse_simple(
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

        let found = find_intersection_removals(&board).unwrap_or(Effects::new());
        assert_eq!(cells!("B8 B9"), found.erases_from_cells(known!(1)));
        assert_eq!(cells!(""), found.erases_from_cells(known!(2)));
        assert_eq!(cells!("D5 E5 F5"), found.erases_from_cells(known!(3)));
        assert_eq!(cells!("D5 E5 F5"), found.erases_from_cells(known!(4)));
        assert_eq!(cells!("B9 C9"), found.erases_from_cells(known!(5)));
        assert_eq!(cells!(""), found.erases_from_cells(known!(6)));
        assert_eq!(cells!(""), found.erases_from_cells(known!(7)));
        assert_eq!(cells!("A5 B5 C5"), found.erases_from_cells(known!(8)));
        assert_eq!(cells!("D1 E1 F1"), found.erases_from_cells(known!(9)));
    }
}
