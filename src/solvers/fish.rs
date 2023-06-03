use super::*;

pub fn find_x_wings(board: &Board) -> Option<Effects> {
    find_fish(board, 2, Strategy::XWing)
}

pub fn find_swordfish(board: &Board) -> Option<Effects> {
    find_fish(board, 3, Strategy::Swordfish)
}

pub fn find_jellyfish(board: &Board) -> Option<Effects> {
    find_fish(board, 4, Strategy::Jellyfish)
}

fn find_fish(board: &Board, size: usize, strategy: Strategy) -> Option<Effects> {
    let mut effects = Effects::new();

    check_houses(board, size, strategy, House::all_rows(), &mut effects);
    check_houses(board, size, strategy, House::all_columns(), &mut effects);

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

fn check_houses(
    board: &Board,
    size: usize,
    strategy: Strategy,
    houses: &[House],
    effects: &mut Effects,
) {
    for k in Known::ALL {
        let candidate_cells = board.candidate_cells(k);
        houses
            .iter()
            .map(|house| (*house, house.cells() & candidate_cells))
            .filter(|(_, cells)| 2 <= cells.size() && cells.size() as usize <= size)
            .map(|(house, cells)| (house, cells, house.crossing_houses(cells)))
            .combinations(size)
            .for_each(|candidates| {
                // HouseSet
                let crosses = candidates.iter().map(|(_, _, crosses)| *crosses).union();
                if crosses.size() as usize != size {
                    return;
                }

                if size > 2
                    && candidates
                        .iter()
                        .map(|(_, _, crosses)| *crosses)
                        .filter(|crosses| crosses.size() < 3)
                        .combinations(2)
                        .map(|pair| pair[0] | pair[1])
                        .any(|union| union.size() <= 2)
                {
                    return;
                }

                if size > 3
                    && candidates
                        .iter()
                        .map(|(_, _, crosses)| *crosses)
                        .filter(|crosses| crosses.size() < 4)
                        .combinations(3)
                        .map(|pair| pair[0] | pair[1] | pair[2])
                        .any(|union| union.size() <= 3)
                {
                    return;
                }

                let main_cells = candidates.iter().map(|(_, cells, _)| *cells).union();
                let cross_cells = crosses.cells() & candidate_cells;
                let erase = cross_cells - main_cells;
                if erase.is_empty() {
                    return;
                }

                let mut action = Action::new(strategy);
                action.erase_cells(erase, k);
                effects.add_action(action);
            });
    }
}
