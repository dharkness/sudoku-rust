use super::*;

pub fn find_x_wings(board: &Board, single: bool) -> Option<Effects> {
    find_fish(board, single, 2, Strategy::XWing)
}

pub fn find_swordfish(board: &Board, single: bool) -> Option<Effects> {
    find_fish(board, single, 3, Strategy::Swordfish)
}

pub fn find_jellyfish(board: &Board, single: bool) -> Option<Effects> {
    find_fish(board, single, 4, Strategy::Jellyfish)
}

fn find_fish(board: &Board, single: bool, size: usize, strategy: Strategy) -> Option<Effects> {
    let mut effects = Effects::new();

    if !check_houses(board, single, size, strategy, Shape::Row, &mut effects) {
        check_houses(board, single, size, strategy, Shape::Column, &mut effects);
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
    size: usize,
    strategy: Strategy,
    shape: Shape,
    effects: &mut Effects,
) -> bool {
    for known in Known::iter() {
        let candidate_cells = board.candidate_cells(known);
        for candidates in shape
            .house_iter()
            .map(|house| (house, house.cells() & candidate_cells))
            .filter(|(_, cells)| 2 <= cells.len() && cells.len() <= size)
            .map(|(house, cells)| (house, cells, house.crossing_houses(cells)))
            .combinations(size)
        {
            let crosses = candidates
                .iter()
                .map(|(_, _, crosses)| *crosses)
                .union_houses();
            if crosses.len() != size {
                continue;
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
                continue;
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
                continue;
            }

            let main_cells = candidates.iter().map(|(_, cells, _)| *cells).union_cells();
            let cross_cells = crosses.cells() & candidate_cells;
            let erase = cross_cells - main_cells;
            if erase.is_empty() {
                continue;
            }

            let mut action = Action::new(strategy);
            action.erase_cells(erase, known);
            candidates.iter().for_each(|(house, cells, _)| {
                action.clue_cells_for_known(Verdict::Secondary, *cells, known);
                action.clue_cells_for_known(
                    Verdict::Related,
                    house.cells() - main_cells - board.knowns(),
                    known,
                );
            });

            if effects.add_action(action) && single {
                return true;
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
    fn x_wing() {
        let board = Parse::packed_with_options(Options::errors()).parse_simple(
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

        let found = find_x_wings(&board, true).unwrap_or(Effects::new());
        assert_eq!(
            cells!("A4 E4 H4 J4 D8 E8 H8 J8"),
            found.erases_from_cells(known!("7"))
        );
    }

    #[test]
    fn swordfish() {
        let board = Parse::packed_with_options(Options::errors()).parse_simple(
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

        let found = find_swordfish(&board, true).unwrap_or(Effects::new());
        assert_eq!(
            cells!("B2 B8 C2 C6 C8 C9 D6"),
            found.erases_from_cells(known!("8"))
        );
    }

    #[test]
    fn jellyfish() {
        let board = Parse::packed_with_options(Options::errors()).parse_simple(
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

        let found = find_jellyfish(&board, true).unwrap_or(Effects::new());
        assert_eq!(
            cells!("B1 B5 B8 C8 C9 G1 G8 H1 H5 H9"),
            found.erases_from_cells(known!("2"))
        );
    }
}
