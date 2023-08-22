use super::*;
use std::collections::{HashMap, HashSet};

pub fn find_unique_rectangles(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    let bi_values =
        board
            .cells_with_n_candidates(2)
            .iter()
            .fold(HashMap::new(), |mut acc, cell| {
                acc.entry(board.candidates(cell))
                    .or_insert(CellSet::empty())
                    .add(cell);
                acc
            });

    for (pair, cells) in bi_values.iter().filter(|(_, cells)| cells.size() >= 2) {
        let (k1, k2) = pair.as_pair().unwrap();
        let mut found_type_ones: HashSet<Rectangle> = HashSet::new();

        for corners in cells.iter().combinations(3).map(CellSet::from_iter) {
            if let Ok(rectangle) = Rectangle::try_from(corners) {
                check_type_one(
                    board,
                    corners,
                    rectangle,
                    *pair,
                    &mut found_type_ones,
                    &mut effects,
                );
            }
        }

        for corners in cells.iter().combinations(2).map(CellSet::from_iter) {
            let (first, second) = corners.as_pair().unwrap();
            let candidates = board.candidate_cells(k1) & board.candidate_cells(k2);

            if first.row() == second.row() {
                for r in House::all_rows() - first.row() {
                    let third = Cell::from_coords(r.coord(), first.column_coord());
                    let fourth = Cell::from_coords(r.coord(), second.column_coord());
                    if !candidates.has(third) || !candidates.has(fourth) {
                        continue;
                    }

                    check_neighbors(
                        board,
                        first,
                        second,
                        third,
                        fourth,
                        *pair,
                        &found_type_ones,
                        &mut effects,
                    )
                }
            } else if first.column() == second.column() {
                for c in House::all_columns() - first.column() {
                    let third = Cell::from_coords(first.row_coord(), c.coord());
                    let fourth = Cell::from_coords(second.row_coord(), c.coord());
                    if !candidates.has(third) || !candidates.has(fourth) {
                        continue;
                    }

                    check_neighbors(
                        board,
                        first,
                        second,
                        third,
                        fourth,
                        *pair,
                        &found_type_ones,
                        &mut effects,
                    )
                }
            } else {
                check_diagonals(board, first, second, *pair, &found_type_ones, &mut effects);
            }
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

fn check_neighbors(
    board: &Board,
    floor_left: Cell,
    floor_right: Cell,
    roof_left: Cell,
    roof_right: Cell,
    pair: KnownSet,
    found_type_ones: &HashSet<Rectangle>,
    effects: &mut Effects,
) {
    let rectangle = Rectangle::from(floor_left, floor_right, roof_left, roof_right);
    if found_type_ones.contains(&rectangle) || rectangle.block_count() > 2 {
        return;
    }

    let roof_left_extras = board.candidates(roof_left) - pair;
    let roof_right_extras = board.candidates(roof_right) - pair;

    check_type_two(
        board,
        rectangle,
        roof_left,
        roof_left_extras,
        roof_right,
        roof_right_extras,
        effects,
    );
    check_type_three(
        board,
        rectangle,
        roof_left,
        roof_left_extras,
        roof_right,
        roof_right_extras,
        effects,
    );
    check_type_four(board, pair, rectangle, roof_left, roof_right, effects);
}

fn check_diagonals(
    board: &Board,
    top: Cell,
    bottom: Cell,
    pair: KnownSet,
    found_type_ones: &HashSet<Rectangle>,
    effects: &mut Effects,
) {
    if let Ok(rectangle) = Rectangle::try_from(CellSet::from_iter([top, bottom])) {
        if rectangle.block_count() > 2 || found_type_ones.contains(&rectangle) {
            return;
        }

        // the floor and roof are formed by the diagonals;
        // the only important part is that both lefts and rights are on the same side.
        let (floor_left, floor_right) = sort_by_column(top, bottom);
        let roof = (rectangle.cells() - CellSet::from_iter([floor_left, floor_right]))
            .as_pair()
            .unwrap();
        let (roof_left, roof_right) = sort_by_column(roof.0, roof.1);

        let roof_left_extras = board.candidates(roof_left) - pair;
        let roof_right_extras = board.candidates(roof_right) - pair;

        // type 5 is related to type 1, and type 4 destroys the unique rectangle
        check_type_five(board, pair, rectangle, floor_left, floor_right, effects);
        check_type_two(
            board,
            rectangle,
            roof_left,
            roof_left_extras,
            roof_right,
            roof_right_extras,
            effects,
        );
        check_type_three(
            board,
            rectangle,
            roof_left,
            roof_left_extras,
            roof_right,
            roof_right_extras,
            effects,
        );
        check_type_four(board, pair, rectangle, roof_left, roof_right, effects);
    }
}

fn sort_by_column(first: Cell, second: Cell) -> (Cell, Cell) {
    if first.column_coord() < second.column_coord() {
        (first, second)
    } else {
        (second, first)
    }
}

fn check_type_one(
    board: &Board,
    corners: CellSet,
    rectangle: Rectangle,
    pair: KnownSet,
    found_type_ones: &mut HashSet<Rectangle>,
    effects: &mut Effects,
) {
    if found_type_ones.contains(&rectangle) || rectangle.block_count() != 2 {
        return;
    }

    let fourth = (rectangle.cells() - corners).as_single().unwrap();
    let candidates = board.candidates(fourth);
    if !candidates.has_all(pair) {
        return;
    }

    found_type_ones.insert(rectangle);
    let mut action = Action::new(Strategy::UniqueRectangle);
    action.erase_knowns(fourth, pair);
    effects.add_action(action);
}

fn check_type_two(
    board: &Board,
    _rectangle: Rectangle,
    roof_left: Cell,
    roof_left_extras: KnownSet,
    roof_right: Cell,
    roof_right_extras: KnownSet,
    effects: &mut Effects,
) {
    if roof_left_extras.size() != 1 || roof_left_extras != roof_right_extras {
        return;
    }

    let extra = roof_left_extras.as_single().unwrap();
    let cells = board.candidate_cells(extra) & roof_left.peers() & roof_right.peers();
    if cells.is_empty() {
        return;
    }

    let mut action = Action::new(Strategy::UniqueRectangle);
    action.erase_cells(cells, extra);
    effects.add_action(action);
}

fn check_type_three(
    board: &Board,
    _rectangle: Rectangle,
    roof_left: Cell,
    roof_left_extras: KnownSet,
    roof_right: Cell,
    roof_right_extras: KnownSet,
    effects: &mut Effects,
) {
    let all_extras = KnownSet::from_iter([roof_left_extras, roof_right_extras]);
    if all_extras.size() != 2 {
        return;
    }

    let (extra1, extra2) = all_extras.as_pair().unwrap();
    let roof = CellSet::from_iter([roof_left, roof_right]);
    let mut erase1 = CellSet::empty();
    let mut erase2 = CellSet::empty();

    for s in Shape::iter() {
        let house = roof_left.house(s);
        if house != roof_right.house(s) {
            continue;
        }

        let peers = house.cells() - roof;
        let naked_candidates =
            CellSet::from_iter(peers.iter().filter(|c| board.candidates(*c) == all_extras));
        if naked_candidates.size() != 1 {
            continue;
        }

        let naked_candidate = naked_candidates.as_single().unwrap();
        erase1 |= board.house_candidate_cells(house, extra1) - roof - naked_candidate;
        erase2 |= board.house_candidate_cells(house, extra2) - roof - naked_candidate;
    }

    if erase1.is_empty() && erase2.is_empty() {
        return;
    }

    let mut action = Action::new(Strategy::UniqueRectangle);
    action.erase_cells(erase1, extra1);
    action.erase_cells(erase2, extra2);
    effects.add_action(action);
}

fn check_type_four(
    board: &Board,
    pair: KnownSet,
    _rectangle: Rectangle,
    roof_left: Cell,
    roof_right: Cell,
    effects: &mut Effects,
) {
    let roof = CellSet::from_iter([roof_left, roof_right]);
    let (pair1, pair2) = pair.as_pair().unwrap();

    for s in Shape::iter() {
        let house = roof_left.house(s);
        if house != roof_right.house(s) {
            continue;
        }

        let pair1_required = board.house_candidate_cells(house, pair1) == roof;
        let pair2_required = board.house_candidate_cells(house, pair2) == roof;
        if pair1_required == pair2_required {
            // not a type 4, and puzzle is invalid if both are required
            continue;
        }

        let erase = if pair1_required { pair2 } else { pair1 };
        let mut action = Action::new(Strategy::UniqueRectangle);
        action.erase_cells(roof, erase);
        effects.add_action(action);
        return;
    }
}

fn check_type_five(
    board: &Board,
    pair: KnownSet,
    _rectangle: Rectangle,
    floor_left: Cell,
    floor_right: Cell,
    effects: &mut Effects,
) {
    let mut erase = None;
    let (pair1, pair2) = pair.as_pair().unwrap();

    for (shape, pair_check, pair_erase) in [
        (Shape::Row, pair1, pair2),
        (Shape::Row, pair2, pair1),
        (Shape::Column, pair1, pair2),
        (Shape::Column, pair2, pair1),
    ] {
        let house_left = floor_left.house(shape);
        let house_right = floor_right.house(shape);
        if board.house_candidate_cells(house_left, pair_check).size() == 2
            && board.house_candidate_cells(house_right, pair_check).size() == 2
        {
            erase = Some(pair_erase);
        }
    }

    if let Some(erase) = erase {
        let mut action = Action::new(Strategy::UniqueRectangle);
        action.erase_cells(CellSet::from_iter([floor_left, floor_right]), erase);
        effects.add_action(action);
    }
}
