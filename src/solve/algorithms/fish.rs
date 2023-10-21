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

    check_houses(board, size, strategy, Shape::Row, &mut effects);
    check_houses(board, size, strategy, Shape::Column, &mut effects);

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
    shape: Shape,
    effects: &mut Effects,
) {
    Known::iter().for_each(|known| {
        let candidate_cells = board.candidate_cells(known);
        shape
            .house_iter()
            .map(|house| (house, house.cells() & candidate_cells))
            .filter(|(_, cells)| 2 <= cells.len() && cells.len() <= size)
            .map(|(house, cells)| (house, cells, house.crossing_houses(cells)))
            .combinations(size)
            .for_each(|candidates| {
                // HouseSet
                let crosses = candidates.iter().map(|(_, _, crosses)| *crosses).union();
                if crosses.len() != size {
                    return;
                }

                if size > 2
                    && candidates
                        .iter()
                        .map(|(_, _, crosses)| *crosses)
                        .filter(|crosses| crosses.len() < 3)
                        .combinations(2)
                        .map(|pair| pair[0] | pair[1])
                        .any(|union| union.len() <= 2)
                {
                    return;
                }

                if size > 3
                    && candidates
                        .iter()
                        .map(|(_, _, crosses)| *crosses)
                        .filter(|crosses| crosses.len() < 4)
                        .combinations(3)
                        .map(|pair| pair[0] | pair[1] | pair[2])
                        .any(|union| union.len() <= 3)
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
                action.erase_cells(erase, known);
                effects.add_action(action);
            });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::Parse;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::values::known::known;

    #[test]
    fn x_wing() {
        let board = Parse::packed_with_options(Options::errors_and_peers()).parse_simple(
            "
                1.....569
                492.561.8
                .561.924.
                ..964.8.1
                .64.1....
                218.356.4
                .4.5...16
                9.5.614.2
                621.....5
            ",
        );

        let found = find_x_wings(&board).unwrap_or(Effects::new());
        assert_eq!(
            cells!("A4 E4 H4 J4 D8 E8 H8 J8"),
            found.erases_from_cells(known!(7))
        );
    }

    #[test]
    fn swordfish() {
        let board = Parse::packed_with_options(Options::errors_and_peers()).parse_simple(
            "
                52941.7.3
                ..6..3..2
                ..32.....
                .523...76
                637.5.2..
                19.62753.
                3...6942.
                2..83.6..
                96.7423.5
            ",
        );

        let found = find_swordfish(&board).unwrap_or(Effects::new());
        assert_eq!(
            cells!("B2 B8 C2 C6 C8 C9 D6"),
            found.erases_from_cells(known!(8))
        );
    }

    #[test]
    fn jellyfish() {
        let board = Parse::packed_with_options(Options::errors_and_peers()).parse_simple(
            "
                ..17538..
                .5......7
                7..89.1..
                ...6.157.
                625478931
                .179.54..
                ....67..4
                .7.....1.
                ..63.97..
            ",
        );

        let found = find_jellyfish(&board).unwrap_or(Effects::new());
        assert_eq!(
            cells!("B1 B5 B8 C8 C9 G1 G8 H1 H5 H9"),
            found.erases_from_cells(known!(2))
        );
    }
}
