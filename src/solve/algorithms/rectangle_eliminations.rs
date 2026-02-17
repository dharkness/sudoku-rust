use super::*;

pub fn find_rectangle_eliminations(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    for digit in Digit::iter() {
        for house in House::rows_iter().chain(House::columns_iter()) {
            let cells = board.house_candidate_cells(house, digit);
            let Some((first, second)) = cells.as_pair() else {
                continue;
            };
            if first.block() == second.block() {
                continue;
            }

            if add_rectangle_eliminations_for_pair(
                board,
                digit,
                house,
                first,
                second,
                &mut effects,
                single,
            ) {
                return Some(effects);
            }
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

fn add_rectangle_eliminations_for_pair(
    board: &Board,
    digit: Digit,
    strong_house: House,
    first: Cell,
    second: Cell,
    effects: &mut Effects,
    single: bool,
) -> bool {
    for (hinge, strong) in [(first, second), (second, first)] {
        if add_rectangle_eliminations_for_hinge(
            board,
            digit,
            strong_house,
            hinge,
            strong,
            effects,
            single,
        ) {
            return true;
        }
    }

    false
}

fn add_rectangle_eliminations_for_hinge(
    board: &Board,
    digit: Digit,
    strong_house: House,
    hinge: Cell,
    strong: Cell,
    effects: &mut Effects,
    single: bool,
) -> bool {
    let weak_house = hinge.house(if strong_house.is_row() {
        Shape::Column
    } else {
        Shape::Row
    });
    let weak_candidates = board.house_candidate_cells(weak_house, digit) - hinge;
    if weak_candidates.is_empty() {
        return false;
    }

    let mut action = Action::new(Strategy::RectangleElimination);
    let mut primary = CellSet::empty();

    for weak in weak_candidates {
        if weak.block() == hinge.block() {
            continue;
        }

        let (row, column) = if strong_house.is_row() {
            (weak.row(), strong.column())
        } else {
            (strong.row(), weak.column())
        };
        let corner = Cell::from_row_column(row, column).block();
        if corner == weak.block() {
            continue;
        }

        let block_cells = board.house_candidate_cells(corner, digit);
        if block_cells.is_empty() {
            continue;
        }
        let allowed = row.cells() | column.cells();
        if !block_cells.is_subset_of(allowed) {
            continue;
        }
        let row_cells = block_cells & row.cells();
        let column_cells = block_cells & column.cells();
        if row_cells.is_empty() || column_cells.is_empty() {
            continue;
        }

        action.erase(weak, digit);
        primary |= block_cells;
    }

    if action.is_empty() {
        return false;
    }

    action.clue_cells_for_digit(Verdict::Primary, primary, digit);
    action.clue_cell_for_digit(Verdict::Secondary, hinge, digit);
    action.clue_cell_for_digit(Verdict::Secondary, strong, digit);

    effects.add_action(action) && single
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
            "S9B027y070n828y8r081m05010804028i8m071i7y067q0n0807057v027y027q060108077y05088e060716027y7q01071601828a03020608017q0508067u0u02070607040203010805097q080282078a1r0n1q",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_rectangle_eliminations(&board, false).expect("not found");
        assert_any_erase(got.actions(), cell!(A2), digit!(9));
    }

    #[test]
    fn example_2() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B027y070n828y8r081m05010804028i8m071i7y067q0n0807057v027y027q060108077y05088e06078a027y7q01078a01828a03020608017q0508067u0u02070607040203010805097q080282078a1r0n1q",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_rectangle_eliminations(&board, false).expect("not found");
        assert_any_erase(got.actions(), cell!(F2), digit!(9));
    }

    #[test]
    fn two_strong_links() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B011m821g2e0u2c82080g1u0h1h091f100c040b037u052i080f019e0h02160i160703060a8i160a1i0226deb69u8i0703080a1u82040b0c010f044i02bm078205b62c2b067n0d440c04b62c2e4m860a4k06",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_rectangle_eliminations(&board, false).expect("not found");
        assert_any_erase(got.actions(), cell!(B7), digit!(5));
        assert_any_erase(got.actions(), cell!(E2), digit!(5));
    }
}
