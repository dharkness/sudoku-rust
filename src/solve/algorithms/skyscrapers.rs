use super::*;

// 697.....2 ..1972.63 ..3..679. 912...6.7 374.6.95. 8657.9.24 148693275 7.9.24..6 ..68.7..9
//
// https://hodoku.sourceforge.net/en/tech_sdp.php
pub fn find_skyscrapers(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    if !check_houses(
        board,
        single,
        House::all_rows(),
        Shape::Column,
        &mut effects,
    ) {
        check_houses(
            board,
            single,
            House::all_columns(),
            Shape::Row,
            &mut effects,
        );
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

fn check_houses(
    board: &Board,
    single: bool,
    houses: HouseSet,
    cross: Shape,
    effects: &mut Effects,
) -> bool {
    for digit in Digit::iter() {
        let candidate_cells = board.candidate_cells(digit);

        let mut check_candidate = |f1: Cell, c1: Cell, f2: Cell, c2: Cell| -> bool {
            if c1.house(cross) == c2.house(cross) {
                // degenerate X-Wing
                return false;
            }
            if (candidate_cells & f1.house(cross).cells()).len() == 2 {
                // degenerate Singles Chain
                return false;
            }

            let candidates = c1.peers() & c2.peers() & candidate_cells;
            if candidates.is_empty() {
                return false;
            }

            let mut action = Action::new(Strategy::Skyscraper);
            action.erase_cells(candidates, digit);
            action.clue_cell_for_digit(Verdict::Secondary, f1, digit);
            action.clue_cell_for_digit(Verdict::Secondary, c2, digit);
            action.clue_cell_for_digit(Verdict::Tertiary, f2, digit);
            action.clue_cell_for_digit(Verdict::Tertiary, c1, digit);

            effects.add_action(action) && single
        };

        for pair in houses
            .iter()
            .map(|house| board.house_candidate_cells(house, digit))
            .filter(|cells| cells.len() == 2)
            .combinations(2)
        {
            let (c11, c12) = pair[0].as_pair().unwrap();
            let (c21, c22) = pair[1].as_pair().unwrap();

            if c11.house(cross) == c21.house(cross) {
                if check_candidate(c11, c12, c21, c22) {
                    return true;
                }
            } else if c11.house(cross) == c22.house(cross) {
                if check_candidate(c11, c12, c22, c21) {
                    return true;
                }
            } else if c12.house(cross) == c21.house(cross) {
                if check_candidate(c12, c11, c21, c22) {
                    return true;
                }
            } else if c12.house(cross) == c22.house(cross) {
                if check_candidate(c12, c11, c22, c21) {
                    return true;
                }
            }
        }
    }

    false
}
