use std::collections::{HashMap, HashSet};

use super::naked_tuples;
use super::*;

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

    for (pair, cells) in bi_values.iter().filter(|(_, cells)| cells.len() >= 2) {
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
    if rectangle.block_count != 2 || found_type_ones.contains(&rectangle) {
        return;
    }

    let fourth = (rectangle.cells - corners).as_single().unwrap();
    let candidates = board.candidates(fourth);
    if !candidates.has_all(pair) {
        return;
    }

    found_type_ones.insert(rectangle);
    let mut action = Action::new(Strategy::UniqueRectangle);
    action.erase_knowns(fourth, pair);
    action.clue_cells_for_knowns(Color::Purple, corners, pair);
    action.clue_cell_for_knowns(Color::Blue, fourth, candidates - pair);

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
    let floor_left_block = floor_left.block();
    let houses = if floor_left_block == floor_right.block() {
        HouseSet::full(shape) - floor_left_block.houses(shape)
    } else {
        floor_left_block.houses(shape) - floor_left.house(shape)
    };

    for house in houses {
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
        if rectangle.block_count != 2 {
            return Err(());
        }

        let (pair1, pair2) = pair.as_pair().unwrap();

        Ok(Self {
            rectangle,
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
        let block1 = floor1.block();
        let block2 = floor2.block();
        if block1 == block2
            || (block1.rows() != block2.rows() && block1.columns() != block2.columns())
        {
            return Err(());
        }

        let floor = CellSet::from_iter([floor1, floor2]);
        let rectangle = Rectangle::try_from(floor)?;
        if rectangle.block_count != 2 {
            return Err(());
        }

        let roof = rectangle.cells - floor;
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
        if self.roof_left_extras.len() != 1 || self.roof_left_extras != self.roof_right_extras {
            return;
        }

        let extra = self.roof_left_extras.as_single().unwrap();
        let cells = board.candidate_cells(extra) & self.roof_left.peers() & self.roof_right.peers();
        if cells.is_empty() {
            return;
        }

        let mut action = Action::new(Strategy::UniqueRectangle);
        action.erase_cells(cells, extra);
        action.clue_cells_for_knowns(Color::Purple, self.rectangle.cells, self.pair);
        action.clue_cells_for_known(Color::Blue, self.roof, extra);
        // println!("type 2 {} - {}", self.rectangle, action);
        effects.add_action(action);
    }

    fn check_type_three(&self, board: &Board, effects: &mut Effects) {
        if !(2..=4).contains(&self.roof_extras.len()) {
            return;
        }

        let mut action = Action::new(Strategy::UniqueRectangle);
        action.clue_cells_for_knowns(Color::Purple, self.rectangle.cells, self.pair);
        action.clue_cell_for_knowns(Color::Blue, self.roof_left, self.roof_left_extras);
        action.clue_cell_for_knowns(Color::Blue, self.roof_right, self.roof_right_extras);

        for house in self.roof_left.common_houses(self.roof_right) {
            let peers = house.cells() - self.roof;
            let peer_knowns: Vec<(Cell, KnownSet)> = peers
                .iter()
                .map(|cell| (cell, board.candidates(cell)))
                .collect();

            for size in 2..=4 {
                // find naked tuples
                if size < self.roof_extras.len() {
                    continue;
                }

                for peer_knowns_combo in peer_knowns
                    .iter()
                    .filter(|(_, knowns)| (2..=size).contains(&knowns.len()))
                    .combinations(size - 1)
                {
                    let known_sets: Vec<KnownSet> = peer_knowns_combo
                        .iter()
                        .map(|(_, ks)| *ks)
                        .chain([self.roof_extras])
                        .collect();
                    let knowns = known_sets.iter().copied().union_knowns();
                    if knowns.len() != size
                        || naked_tuples::is_degenerate(&known_sets, size, 2)
                        || naked_tuples::is_degenerate(&known_sets, size, 3)
                    {
                        continue;
                    }

                    let cells = peers - peer_knowns_combo.iter().map(|(c, _)| *c).union_cells();

                    for (cell, knowns) in peer_knowns_combo {
                        action.clue_cell_for_knowns(Color::Blue, *cell, *knowns);
                    }
                    for known in knowns {
                        action.erase_cells(cells & board.candidate_cells(known), known)
                    }
                    break;
                }
            }
        }

        // println!("type 3 {} - {}", self.rectangle, action);
        effects.add_action(action);
    }

    fn check_type_four(&self, board: &Board, effects: &mut Effects) {
        for shape in Shape::iter() {
            let house = self.roof_left.house(shape);
            if house != self.roof_right.house(shape) {
                continue;
            }

            let pair1_required = board.house_candidate_cells(house, self.pair1) == self.roof;
            let pair2_required = board.house_candidate_cells(house, self.pair2) == self.roof;
            if pair1_required == pair2_required {
                // not a type 4, and puzzle is invalid if both are required
                continue;
            }

            let (required, erase) = if pair1_required {
                (self.pair1, self.pair2)
            } else {
                (self.pair2, self.pair1)
            };
            let mut action = Action::new(Strategy::UniqueRectangle);
            action.erase_cells(self.roof, erase);
            action.clue_cells_for_knowns(Color::Purple, self.floor, self.pair);
            action.clue_cells_for_known(Color::Blue, self.roof, required);

            // println!("type 4 {} - {}", self.rectangle, action);
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
            if board.house_candidate_cells(house_left, pair_check).len() == 2
                && board.house_candidate_cells(house_right, pair_check).len() == 2
            {
                erase = Some(pair_erase);
            }
        }

        if let Some(erase) = erase {
            let mut action = Action::new(Strategy::UniqueRectangle);
            action.erase_cells(CellSet::of(&[self.floor_left, self.floor_right]), erase);
            action.clue_cells_for_knowns(Color::Purple, self.roof, self.pair);
            action.clue_cells_for_knowns(Color::Purple, self.floor, self.pair - erase);

            // println!("type 5 {} - {}", self.rectangle, action);
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

#[cfg(test)]
mod tests {
    use crate::io::{Parse, Parser};
    use crate::layout::cells::cell::cell;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::values::known::known;
    use crate::layout::values::known_set::knowns;

    use super::*;

    #[test]
    fn test_type_1() {
        let parser = Parse::wiki();
        let (board, effects, failed) = parser.parse(
            "k0k02109050h81031181110c21g1030k410sgkgs03418111gki8ish6g60hh009412181g40981h0h02105030h41g421410h03810911g4jkgkh4034109hgi0815048h8810h21h005032i0q810511g141282o",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_unique_rectangles(&board) {
            let mut action =
                Action::new_erase_knowns(Strategy::UniqueRectangle, cell!("D1"), knowns!("2 9"));
            action.clue_cells_for_knowns(Color::Purple, cells!("D9 F1 F9"), knowns!("2 9"));
            action.clue_cell_for_knowns(Color::Blue, cell!("D1"), knowns!("1 5"));
            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }

    #[test]
    fn test_type_2() {
        let parser = Parse::packed_with_options(Options::all());
        let (board, effects, failed) = parser.parse(
            "42.9..386 .6.2..794 8.9.6.251 7....3.25 9..1.26.3 2..5....8 ..4.2.567 6827..439 ......812",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_unique_rectangles(&board) {
            let mut action =
                Action::new_erase_cells(Strategy::UniqueRectangle, cells!("A3 C6"), known!("7"));
            action.clue_cells_for_knowns(Color::Purple, cells!("A5 A6 H5 H6"), knowns!("1 5"));
            action.clue_cells_for_known(Color::Blue, cells!("A5 A6"), known!("7"));
            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }

    #[test]
    fn test_type_2_diagonal() {
        let parser = Parse::wiki();
        let (board, effects, failed) = parser.parse(
            "814kg10s2u246c116e110922812m41i42mg42i4k621sg134812m6e05g10h215081030950418128g11c0334240h2803114c4c0h64g181gq4g055g81j0j822jagg1181032k09g441i4ga214a5454h40h81he",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_unique_rectangles(&board) {
            let mut action =
                Action::new_erase_cells(Strategy::UniqueRectangle, cells!("A9 C9 G7"), known!("6"));
            action.clue_cells_for_knowns(Color::Purple, cells!("B7 B9 H7 H9"), knowns!("2 9"));
            action.clue_cells_for_known(Color::Blue, cells!("B7 H9"), known!("6"));
            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }

    #[test]
    fn test_type_3() {
        let parser = Parse::wiki();
        let (board, effects, failed) = parser.parse(
            "gg800541081102igiggi4111g2210408gg81g209200go2o00540100812g08104607g3i2i21040h10k002k08009811240g80gi8j0j20440g18a20928o1g050i05210agal2ko80hqgi100g8a05o2o8i0ia40",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_unique_rectangles(&board) {
            let mut action = Action::new(Strategy::UniqueRectangle);
            action.erase_knowns(cell!("H8"), knowns!("4 9"));
            action.erase_knowns(cell!("J8"), knowns!("6 9"));
            action.clue_cells_for_knowns(Color::Purple, cells!("D2 D8 F2 F8"), knowns!("1 5"));
            action.clue_cell_for_knowns(Color::Blue, cell!("A8"), knowns!("4 6 9"));
            action.clue_cell_for_knowns(Color::Blue, cell!("B8"), knowns!("4 9"));
            action.clue_cell_for_knowns(Color::Blue, cell!("D8"), knowns!("4 6"));
            action.clue_cell_for_knowns(Color::Blue, cell!("F8"), knowns!("6 9"));
            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }

    #[test]
    fn test_type_4() {
        let parser = Parse::wiki();
        let (board, effects, failed) = parser.parse(
            "k0k02109050h81031181110c21g1030k410sgkgs03418111gki8is12g60hh009412181g40981h0h02105030h41g421410h03810911g4jkgkh4034109hgi081l0k8h8810h21h005032i0q810511g141282o",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_unique_rectangles(&board) {
            let mut action = Action::new(Strategy::UniqueRectangle);
            action.erase_cells(cells!("H1 H2"), known!("9"));
            action.clue_cells_for_knowns(Color::Purple, cells!("A1 A2"), knowns!("7 9"));
            action.clue_cells_for_known(Color::Blue, cells!("H1 H2"), known!("7"));
            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }

    #[test]
    fn test_type_5() {
        let parser = Parse::grid().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "
                +----------------+----------------+-----------------+
                | 7   589  3589  | 4   259  6     | 28    238  1    |
                | 49  2    369   | 8   179  17    | 467   367  5    |
                | 1   4568 568   | 3   257  27    | 24678 9    2467 |
                +----------------+----------------+-----------------+
                | 3   169  4     | 279 127  5     | 2678  2678 267  |
                | 289 7    56    | 29  3    28    | 456   1    46   |
                | 28  15   125   | 6   1247 12478 | 3     257  9    |
                +----------------+----------------+-----------------+
                | 249 3    1279  | 5   2467 247   | 19    267  8    |
                | 5   189  12789 | 27  2678 3     | 19    4    267  |
                | 6   48   278   | 1   2478 9     | 257   257  3    |
                +----------------+----------------+-----------------+
            ",
        );
        assert!(failed.is_none());
        assert!(!effects.has_errors());

        if let Some(got) = find_unique_rectangles(&board) {
            let mut action =
                Action::new_erase_cells(Strategy::UniqueRectangle, cells!("E6 F1"), known!("2"));
            action.clue_cells_for_known(Color::Purple, cells!("E1 F6"), known!("2"));
            action.clue_cells_for_known(Color::Purple, cells!("E1 E6 F1 F6"), known!("8"));
            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }
}
