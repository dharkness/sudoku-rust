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

            if first.row() == second.row() {
                check_neighbors(
                    board,
                    *pair,
                    first,
                    second,
                    Shape::Row,
                    &found_type_ones,
                    &mut effects,
                );
            } else if first.column() == second.column() {
                check_neighbors(
                    board,
                    *pair,
                    first,
                    second,
                    Shape::Column,
                    &found_type_ones,
                    &mut effects,
                );
            } else {
                check_diagonals(board, *pair, first, second, &found_type_ones, &mut effects);
            }
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
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

fn check_neighbors(
    board: &Board,
    pair: KnownSet,
    floor_left: Cell,
    floor_right: Cell,
    shape: Shape,
    found_type_ones: &HashSet<Rectangle>,
    effects: &mut Effects,
) {
    for house in HouseSet::full(shape) - floor_left.house(shape) {
        if let Ok(candidate) =
            Candidate::try_from_neighbors(board, pair, floor_left, floor_right, house)
        {
            if !found_type_ones.contains(&candidate.rectangle) {
                candidate.check(board, effects);
            }
        }
    }
}

fn check_diagonals(
    board: &Board,
    pair: KnownSet,
    top: Cell,
    bottom: Cell,
    found_type_ones: &HashSet<Rectangle>,
    effects: &mut Effects,
) {
    if let Ok(candidate) = Candidate::try_from_diagonals(board, pair, top, bottom) {
        if !found_type_ones.contains(&candidate.rectangle) {
            candidate.check(board, effects);
        }
    }
}

struct Candidate {
    pub rectangle: Rectangle,
    pub block_count: usize,
    pub pair: KnownSet,
    pub pair1: Known,
    pub pair2: Known,

    pub diagonal: bool,
    pub floor: CellSet,
    pub floor_left: Cell,
    pub floor_right: Cell,

    pub roof: CellSet,
    pub roof_extras: KnownSet,
    pub roof_left: Cell,
    pub roof_left_extras: KnownSet,
    pub roof_right: Cell,
    pub roof_right_extras: KnownSet,
}

impl Candidate {
    fn try_from_neighbors(
        board: &Board,
        pair: KnownSet,
        floor_left: Cell,
        floor_right: Cell,
        roof_house: House,
    ) -> Result<Self, ()> {
        match roof_house.shape() {
            Shape::Row => Self::try_from_corners(
                board,
                pair,
                floor_left,
                floor_right,
                Cell::from_coords(roof_house.coord(), floor_left.column_coord()),
                Cell::from_coords(roof_house.coord(), floor_right.column_coord()),
            ),
            Shape::Column => Self::try_from_corners(
                board,
                pair,
                floor_left,
                floor_right,
                Cell::from_coords(floor_left.row_coord(), roof_house.coord()),
                Cell::from_coords(floor_right.row_coord(), roof_house.coord()),
            ),
            Shape::Block => Err(()),
        }
    }

    fn try_from_corners(
        board: &Board,
        pair: KnownSet,
        floor_left: Cell,
        floor_right: Cell,
        roof_left: Cell,
        roof_right: Cell,
    ) -> Result<Self, ()> {
        let roof_left_candidates = board.candidates(roof_left);
        if !roof_left_candidates.has_all(pair) {
            return Err(());
        }
        let roof_right_candidates = board.candidates(roof_right);
        if !roof_right_candidates.has_all(pair) {
            return Err(());
        }
        let roof_left_extras = board.candidates(roof_left) - pair;
        let roof_right_extras = board.candidates(roof_right) - pair;

        let rectangle = Rectangle::from(floor_left, floor_right, roof_left, roof_right);
        if rectangle.block_count() > 2 {
            return Err(());
        }

        let (pair1, pair2) = pair.as_pair().unwrap();

        Ok(Self {
            rectangle,
            block_count: rectangle.block_count(),
            pair,
            pair1,
            pair2,
            diagonal: false,
            floor: CellSet::from_iter([floor_left, floor_right]),
            floor_left,
            floor_right,
            roof: CellSet::from_iter([roof_left, roof_right]),
            roof_extras: roof_left_extras | roof_right_extras,
            roof_left,
            roof_left_extras,
            roof_right,
            roof_right_extras,
        })
    }

    fn try_from_diagonals(
        board: &Board,
        pair: KnownSet,
        floor1: Cell,
        floor2: Cell,
    ) -> Result<Self, ()> {
        let floor = CellSet::from_iter([floor1, floor2]);
        let rectangle = Rectangle::try_from(floor)?;
        if rectangle.block_count() > 2 {
            return Err(());
        }

        let roof = rectangle.cells() - floor;
        let roof_pair = roof.as_pair().unwrap();

        // the floor and roof are formed by the diagonals;
        // the only important part is that lefts and rights are on the same side.
        let (floor_left, floor_right) = sort_by_column(floor1, floor2);
        let (roof_left, roof_right) = sort_by_column(roof_pair.0, roof_pair.1);

        let roof_left_candidates = board.candidates(roof_left);
        if !roof_left_candidates.has_all(pair) {
            return Err(());
        }
        let roof_right_candidates = board.candidates(roof_right);
        if !roof_right_candidates.has_all(pair) {
            return Err(());
        }
        let roof_left_extras = board.candidates(roof_left) - pair;
        let roof_right_extras = board.candidates(roof_right) - pair;

        let (pair1, pair2) = pair.as_pair().unwrap();

        Ok(Self {
            rectangle,
            block_count: rectangle.block_count(),
            pair,
            pair1,
            pair2,
            diagonal: true,
            floor,
            floor_left,
            floor_right,
            roof,
            roof_extras: roof_left_extras | roof_right_extras,
            roof_left,
            roof_left_extras,
            roof_right,
            roof_right_extras,
        })
    }

    fn check(&self, board: &Board, effects: &mut Effects) {
        if self.diagonal {
            // type 5 is related to type 1, and type 4 destroys the unique rectangle
            self.check_type_five(board, effects);
        }
        self.check_type_two(board, effects);
        self.check_type_three(board, effects);
        self.check_type_four(board, effects);
    }

    fn check_type_two(&self, board: &Board, effects: &mut Effects) {
        if self.roof_left_extras.size() != 1 || self.roof_left_extras != self.roof_right_extras {
            return;
        }

        let extra = self.roof_left_extras.as_single().unwrap();
        let cells = board.candidate_cells(extra) & self.roof_left.peers() & self.roof_right.peers();
        if cells.is_empty() {
            return;
        }

        let mut action = Action::new(Strategy::UniqueRectangle);
        action.erase_cells(cells, extra);
        effects.add_action(action);
    }

    // TODO extend to triples and quads
    fn check_type_three(&self, board: &Board, effects: &mut Effects) {
        if self.roof_extras.size() != 2 {
            return;
        }

        let (extra1, extra2) = self.roof_extras.as_pair().unwrap();
        let mut erase1 = CellSet::empty();
        let mut erase2 = CellSet::empty();

        for s in Shape::iter() {
            let house = self.roof_left.house(s);
            if house != self.roof_right.house(s) {
                continue;
            }

            let peers = house.cells() - self.roof;
            let naked_candidates = CellSet::from_iter(
                peers
                    .iter()
                    .filter(|c| board.candidates(*c) == self.roof_extras),
            );
            if naked_candidates.size() != 1 {
                continue;
            }

            let naked_candidate = naked_candidates.as_single().unwrap();
            erase1 |= board.house_candidate_cells(house, extra1) - self.roof - naked_candidate;
            erase2 |= board.house_candidate_cells(house, extra2) - self.roof - naked_candidate;
        }

        if erase1.is_empty() && erase2.is_empty() {
            return;
        }

        let mut action = Action::new(Strategy::UniqueRectangle);
        action.erase_cells(erase1, extra1);
        action.erase_cells(erase2, extra2);
        effects.add_action(action);
    }

    fn check_type_four(&self, board: &Board, effects: &mut Effects) {
        for s in Shape::iter() {
            let house = self.roof_left.house(s);
            if house != self.roof_right.house(s) {
                continue;
            }

            let pair1_required = board.house_candidate_cells(house, self.pair1) == self.roof;
            let pair2_required = board.house_candidate_cells(house, self.pair2) == self.roof;
            if pair1_required == pair2_required {
                // not a type 4, and puzzle is invalid if both are required
                continue;
            }

            let erase = if pair1_required {
                self.pair2
            } else {
                self.pair1
            };
            let mut action = Action::new(Strategy::UniqueRectangle);
            action.erase_cells(self.roof, erase);
            effects.add_action(action);
            return;
        }
    }

    fn check_type_five(&self, board: &Board, effects: &mut Effects) {
        let mut erase = None;

        for (shape, pair_check, pair_erase) in [
            (Shape::Row, self.pair1, self.pair2),
            (Shape::Row, self.pair2, self.pair1),
            (Shape::Column, self.pair1, self.pair2),
            (Shape::Column, self.pair2, self.pair1),
        ] {
            let house_left = self.floor_left.house(shape);
            let house_right = self.floor_right.house(shape);
            if board.house_candidate_cells(house_left, pair_check).size() == 2
                && board.house_candidate_cells(house_right, pair_check).size() == 2
            {
                erase = Some(pair_erase);
            }
        }

        if let Some(erase) = erase {
            let mut action = Action::new(Strategy::UniqueRectangle);
            action.erase_cells(
                CellSet::from_iter([self.floor_left, self.floor_right]),
                erase,
            );
            effects.add_action(action);
        }
    }
}

fn sort_by_column(first: Cell, second: Cell) -> (Cell, Cell) {
    if first.column_coord() < second.column_coord() {
        (first, second)
    } else {
        (second, first)
    }
}
