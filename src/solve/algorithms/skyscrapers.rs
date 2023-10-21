use super::*;

pub fn find_skyscrapers(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    check_houses(board, House::all_rows(), Shape::Column, &mut effects);
    check_houses(board, House::all_columns(), Shape::Row, &mut effects);

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

fn check_houses(board: &Board, houses: HouseSet, cross: Shape, effects: &mut Effects) {
    for known in Known::iter() {
        let mut check_candidate = |f1: Cell, c1: Cell, _f2: Cell, c2: Cell| {
            if c1.house(cross) == c2.house(cross) {
                // degenerate X-Wing
                return;
            }
            if (board.candidate_cells(known) & f1.house(cross).cells()).len() == 2 {
                // degenerate Singles Chain
                return;
            }

            let candidates = c1.peers() & c2.peers() & board.candidate_cells(known);
            if candidates.is_empty() {
                return;
            }

            let mut action = Action::new(Strategy::Skyscraper);
            action.erase_cells(candidates, known);
            effects.add_action(action);
        };

        houses
            .iter()
            .map(|house| board.house_candidate_cells(house, known))
            .filter(|cells| cells.len() == 2)
            .combinations(2)
            .for_each(|pair| {
                let (c11, c12) = pair[0].as_pair().unwrap();
                let (c21, c22) = pair[1].as_pair().unwrap();

                if c11.house(cross) == c21.house(cross) {
                    check_candidate(c11, c12, c21, c22);
                } else if c11.house(cross) == c22.house(cross) {
                    check_candidate(c11, c12, c22, c21);
                } else if c12.house(cross) == c21.house(cross) {
                    check_candidate(c12, c11, c21, c22);
                } else if c12.house(cross) == c22.house(cross) {
                    check_candidate(c12, c11, c22, c21);
                }
            });
    }
}
