use itertools::Itertools;

use super::*;

pub fn find_sue_de_coq(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    if !check_shape(board, Shape::Row, single, &mut effects) {
        check_shape(board, Shape::Column, single, &mut effects);
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

fn check_shape(board: &Board, shape: Shape, single: bool, effects: &mut Effects) -> bool {
    for block in House::blocks_iter() {
        for line in match shape {
            Shape::Row => block.row_iter().collect::<Vec<_>>(),
            Shape::Column => block.column_iter().collect::<Vec<_>>(),
            Shape::Block => continue,
        } {
            if check_block_line(board, block, line, single, effects) {
                return true;
            }
        }
    }

    false
}

fn check_block_line(
    board: &Board,
    block: House,
    line: House,
    single: bool,
    effects: &mut Effects,
) -> bool {
    let intersection = block.intersect(line);
    let inter_cells: Vec<Cell> = intersection
        .iter()
        .filter(|cell| board.candidates(*cell).len() >= 2)
        .collect();
    if inter_cells.len() < 2 {
        return false;
    }

    let line_outside = line.cells() - block.cells();
    let box_outside = block.cells() - line.cells();
    if line_outside.is_empty() || box_outside.is_empty() {
        return false;
    }

    let line_als = find_als_in_cells(board, line_outside);
    let box_als = find_als_in_cells(board, box_outside);
    if line_als.is_empty() || box_als.is_empty() {
        return false;
    }

    for size in 2..=3 {
        if inter_cells.len() < size {
            continue;
        }

        for combo in inter_cells.iter().copied().combinations(size) {
            let mut core_cells = CellSet::empty();
            let mut core_digits = DigitSet::empty();
            for cell in &combo {
                core_cells.add(*cell);
                core_digits |= board.candidates(*cell);
            }

            if core_digits.len() < size + 2 {
                continue;
            }

            for line_als_set in &line_als {
                if !line_als_set.digits.is_subset_of(core_digits) {
                    continue;
                }
                for box_als_set in &box_als {
                    if !box_als_set.digits.is_subset_of(core_digits) {
                        continue;
                    }
                    if !(line_als_set.digits & box_als_set.digits).is_empty() {
                        continue;
                    }

                    let total_cells = size + line_als_set.cells.len() + box_als_set.cells.len();
                    if core_digits.len() != total_cells {
                        continue;
                    }

                    let mut action = Action::new(Strategy::SueDeCoq);
                    let line_targets = line.cells() - core_cells - line_als_set.cells;
                    for digit in line_als_set.digits {
                        let erase = line_targets & board.candidate_cells(digit);
                        if !erase.is_empty() {
                            action.erase_cells(erase, digit);
                        }
                    }

                    let box_targets = block.cells() - core_cells - box_als_set.cells;
                    for digit in box_als_set.digits {
                        let erase = box_targets & board.candidate_cells(digit);
                        if !erase.is_empty() {
                            action.erase_cells(erase, digit);
                        }
                    }

                    if action.is_empty() {
                        continue;
                    }

                    action.clue_cells_for_digits(Verdict::Primary, core_cells, core_digits);
                    action.clue_cells_for_digits(
                        Verdict::Secondary,
                        line_als_set.cells,
                        line_als_set.digits,
                    );
                    action.clue_cells_for_digits(
                        Verdict::Secondary,
                        box_als_set.cells,
                        box_als_set.digits,
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

#[derive(Clone)]
struct Als {
    cells: CellSet,
    digits: DigitSet,
}

fn find_als_in_cells(board: &Board, cells: CellSet) -> Vec<Als> {
    let cell_vec: Vec<Cell> = cells
        .iter()
        .filter(|cell| board.candidates(*cell).len() >= 2)
        .collect();
    let cell_count = cell_vec.len();
    if cell_count == 0 {
        return Vec::new();
    }

    let candidates: Vec<DigitSet> = cell_vec
        .iter()
        .map(|cell| board.candidates(*cell))
        .collect();
    let mut als_sets = Vec::new();
    let total_masks = 1usize << cell_count;
    for mask in 1..total_masks {
        let size = mask.count_ones() as usize;
        let mut digits = DigitSet::empty();
        let mut subset_cells = CellSet::empty();
        for (index, cell_digits) in candidates.iter().enumerate() {
            if (mask & (1usize << index)) != 0 {
                digits |= *cell_digits;
                subset_cells.add(cell_vec[index]);
            }
        }

        if digits.len() == size + 1 {
            als_sets.push(Als {
                cells: subset_cells,
                digits,
            });
        }
    }

    als_sets
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
            "S9B2i010u0905081q3e02b844060304070501b6055ybi0602014ed65ub94k074b0610bf03b70648bd635u0o4bd6054304125v0912026q6r5x784l5u03094z50044c094c050106074403036q43025u04c305c3",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_sue_de_coq(&board, false).expect("not found");
        assert_any_erase(got.actions(), cell!(C2), digit!(8));
        assert_any_erase(got.actions(), cell!(J2), digit!(8));
        assert_any_erase(got.actions(), cell!(G2), digit!(2));
        assert_any_erase(got.actions(), cell!(E3), digit!(3));
    }

    #[test]
    fn example_2() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B01055u0403025u0609092c040108060o2g051g50038207824545043m8i025i1f6bc6042f3u4y4j1209331j5z023e03b74y026205cz2b109g82c61fbb042f2f03040607057n7p7p0808019e02047q9i0506",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_sue_de_coq(&board, false).expect("not found");
        assert_any_erase(got.actions(), cell!(E1), digit!(6));
        assert_any_erase(got.actions(), cell!(E3), digit!(8));
        assert_any_erase(got.actions(), cell!(D7), digit!(3));
        assert_any_erase(got.actions(), cell!(F8), digit!(1));
        assert_any_erase(got.actions(), cell!(F8), digit!(7));
    }
}
