use std::collections::{HashMap, HashSet};

use super::naked_tuples;
use super::*;

pub fn find_unique_rectangles(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    let bi_values_by_candidates = board.cell_candidates_with_n_candidates(2).fold(
        HashMap::new(),
        |mut map: HashMap<KnownSet, CellSet>, (cell, candidates)| {
            *map.entry(candidates).or_default() += cell;
            map
        },
    );

    for (pair, cells) in bi_values_by_candidates
        .iter()
        .filter(|(_, cells)| cells.len() >= 2)
    {
        let mut found_type_ones: HashSet<Rectangle> = HashSet::new();

        for corners in cells.iter().combinations(3).map(CellSet::from_iter) {
            if let Ok(rectangle) = Rectangle::try_from(corners) {
                if check_type_one(
                    board,
                    single,
                    corners,
                    rectangle,
                    *pair,
                    &mut found_type_ones,
                    &mut effects,
                ) {
                    return Some(effects);
                }
            }
        }

        for corners in cells.iter().combinations(2).map(CellSet::from_iter) {
            let (first, second) = corners.as_pair().unwrap();

            if first.row() == second.row() {
                if check_neighbors(
                    board,
                    single,
                    *pair,
                    first,
                    second,
                    Shape::Row,
                    &found_type_ones,
                    &mut effects,
                ) {
                    return Some(effects);
                }
            } else if first.column() == second.column() {
                if check_neighbors(
                    board,
                    single,
                    *pair,
                    first,
                    second,
                    Shape::Column,
                    &found_type_ones,
                    &mut effects,
                ) {
                    return Some(effects);
                }
            } else {
                if check_diagonals(
                    board,
                    single,
                    *pair,
                    first,
                    second,
                    &found_type_ones,
                    &mut effects,
                ) {
                    return Some(effects);
                }
            }
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

/// When three corners contain only the pair, the pair may be removed from the fourth corner.
///
/// ```
///    1   2   3     4   5   6
///   ··· ··· ··· | ··· ··· ···
/// A ··· ·5· ··· | ··· ·5· ···
///   ··· ··9 ··· | ··· ··9 ···
///               |
///   ··· ··· ··· | ··· ·2· ···
/// B ··· ·5· ··· | ··· 45· ···  ←-- cell B5 may not contain 5 or 9,
///   ··· ··9 ··· | ··· ··9 ···      and so they may be removed
/// ```
fn check_type_one(
    board: &Board,
    single: bool,
    corners: CellSet,
    rectangle: Rectangle,
    pair: KnownSet,
    found_type_ones: &mut HashSet<Rectangle>,
    effects: &mut Effects,
) -> bool {
    if rectangle.block_count != 2 || found_type_ones.contains(&rectangle) {
        return false;
    }

    let fourth = (rectangle.cells - corners).as_single().unwrap();
    let candidates = board.candidates(fourth);
    let erase = candidates & pair;
    if erase.is_empty() {
        return false;
    }

    found_type_ones.insert(rectangle);
    let mut action = Action::new(if erase.len() == 2 {
        Strategy::UniqueRectangle
    } else {
        Strategy::AlmostUniqueRectangle
    });
    action.erase_knowns(fourth, erase);
    action.clue_cells_for_knowns(Verdict::Primary, corners, pair);
    action.clue_cell_for_knowns(Verdict::Secondary, fourth, candidates - pair);

    effects.add_action(action) && single
}

fn check_neighbors(
    board: &Board,
    single: bool,
    pair: KnownSet,
    floor_left: Cell,
    floor_right: Cell,
    shape: Shape,
    found_type_ones: &HashSet<Rectangle>,
    effects: &mut Effects,
) -> bool {
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
                if candidate.check(board, single, effects) {
                    return true;
                }
            }
        }
    }

    false
}

fn check_diagonals(
    board: &Board,
    single: bool,
    pair: KnownSet,
    top: Cell,
    bottom: Cell,
    found_type_ones: &HashSet<Rectangle>,
    effects: &mut Effects,
) -> bool {
    if let Ok(candidate) = Candidate::try_from_diagonals(board, pair, top, bottom) {
        if !found_type_ones.contains(&candidate.rectangle) {
            if candidate.check(board, single, effects) {
                return true;
            }
        }
    }

    false
}

struct Candidate {
    /// The raw rectangle with the floor and roof cells.
    pub rectangle: Rectangle,
    /// The pair of candidates in danger of forming a deadly rectangle.
    pub pair: KnownSet,
    pub pair1: Known,
    pub pair2: Known,

    /// True if the two floor cells are diagonally opposite corners.
    pub diagonal: bool,
    pub floor: CellSet,
    pub floor_left: Cell,
    pub floor_right: Cell,

    pub roof: CellSet,
    pub roof_has_any: bool,
    pub roof_has_both: bool,
    pub roof_extras: KnownSet,
    pub roof_left: Cell,
    pub roof_left_pair: KnownSet,
    pub roof_left_extras: KnownSet,
    pub roof_right: Cell,
    pub roof_right_pair: KnownSet,
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
        if !roof_left_candidates.has_any(pair) {
            return Err(());
        }
        // if !roof_left_candidates.has_all(pair) {
        //     return Err(());
        // }
        let roof_right_candidates = board.candidates(roof_right);
        if !roof_right_candidates.has_any(pair) {
            return Err(());
        }
        // if !roof_right_candidates.has_all(pair) {
        //     return Err(());
        // }
        let roof_common_candidates = roof_left_candidates & roof_right_candidates;
        let roof_all_candidates = roof_left_candidates | roof_right_candidates;
        if !roof_all_candidates.has_all(pair) {
            return Err(());
        }

        let roof_left_extras = roof_left_candidates - pair;
        let roof_right_extras = roof_right_candidates - pair;

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
            roof_has_any: roof_common_candidates.has_any(pair),
            roof_has_both: roof_common_candidates.has_all(pair),
            roof_extras: roof_left_extras | roof_right_extras,
            roof_left,
            roof_left_pair: roof_left_candidates & pair,
            roof_left_extras,
            roof_right,
            roof_right_pair: roof_right_candidates & pair,
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
        if !roof_left_candidates.has_any(pair) {
            return Err(());
        }
        // if !roof_left_candidates.has_all(pair) {
        //     return Err(());
        // }
        let roof_right_candidates = board.candidates(roof_right);
        if !roof_right_candidates.has_any(pair) {
            return Err(());
        }
        // if !roof_right_candidates.has_all(pair) {
        //     return Err(());
        // }
        let roof_left_extras = roof_left_candidates - pair;
        let roof_right_extras = roof_right_candidates - pair;
        let roof_candidates = roof_left_candidates & roof_right_candidates;

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
            roof_has_any: roof_candidates.has_any(pair),
            roof_has_both: roof_candidates.has_all(pair),
            roof_extras: roof_left_extras | roof_right_extras,
            roof_left,
            roof_left_pair: roof_left_candidates & pair,
            roof_left_extras,
            roof_right,
            roof_right_pair: roof_right_candidates & pair,
            roof_right_extras,
        })
    }

    fn add_clues_for_all_corner_cells(&self, action: &mut Action) {
        action.clue_cell_for_knowns(Verdict::Primary, self.floor_left, self.pair);
        action.clue_cell_for_knowns(Verdict::Primary, self.floor_right, self.pair);
        action.clue_cell_for_knowns(Verdict::Primary, self.roof_left, self.roof_left_pair);
        action.clue_cell_for_knowns(Verdict::Primary, self.roof_right, self.roof_right_pair);
    }

    fn check(&self, board: &Board, single: bool, effects: &mut Effects) -> bool {
        // type 5 is related to type 1, and type 4 destroys the unique rectangle
        if self.diagonal && self.check_type_five(board, effects) && single {
            return true;
        }
        if self.check_type_two(board, effects) && single {
            return true;
        }
        if !self.diagonal && self.check_type_three(board, effects) && single {
            return true;
        }
        if !self.diagonal && self.check_type_four(board, effects) && single {
            return true;
        }

        false
    }

    /// Two neighbors share the bi-value pair while the other two share one additional candidate.
    /// The candidate may be removed from all cells that see both neighbors that share it.
    ///
    /// ```
    ///    1   2   3     4   5   6     7   8   9
    ///   ··· ··· ··· | ··· ··· ··· | ··· ··· ···
    /// A ··· ·5· ··· | ··· ··· ··· | ··· ·5· ···
    ///   ··· ··9 ··· | ··· ··· ··· | ··· ··9 ···
    ///               |             |
    ///   ·2· ·2· ··· | ··· ·2· ··· | ··· ·2· ···
    /// B ··· ·5· ··· | ··· ··· ··· | ··· ·5· ···  ←-- 2 must appear in cell B2 or B8,
    ///   ··· ··9 ··· | ··· ··· ··· | ··· ··9 ···      and so it may be removed from cells B1 and B5
    /// ```
    fn check_type_two(&self, board: &Board, effects: &mut Effects) -> bool {
        if self.roof_left_extras != self.roof_right_extras || self.roof_left_extras.len() != 1 {
            return false;
        }

        let extra = self.roof_left_extras.as_single().unwrap();
        let cells = board.candidate_cells(extra) & self.roof_left.peers() & self.roof_right.peers();
        if cells.is_empty() {
            return false;
        }

        let mut action = Action::new(if self.roof_has_both {
            Strategy::UniqueRectangle
        } else {
            Strategy::AlmostUniqueRectangle
        });
        action.erase_cells(cells, extra);
        self.add_clues_for_all_corner_cells(&mut action);
        action.clue_cells_for_known(Verdict::Secondary, self.roof, extra);
        // println!("type 2 {} - {}", self.rectangle, action);

        effects.add_action(action)
    }

    /// When the two non-bi-value neighbors share one or two additional candidates,
    /// and they see another cell with only those two candidates, they neighbors
    /// form a pseudo cell and a naked pair with the third cell.
    ///
    /// This also applies to naked triples and quads with three and four additional candidates,
    /// respectively.
    ///
    /// ```
    ///    1   2   3     4   5   6     7   8   9
    ///   ··· ··· ··· | ··· ··· ··· | ··· 12· ·2·
    /// A ··· ·5· ··· | ··· ··· ··· | ··· ·5· 4··
    ///   ··· ··9 ··· | ··· ··· ··· | ··· ··9 ·89
    ///               |             |
    ///   ··· ··· ··· | ··· ··· ··· | 1·3 ·2· 12·  ←-- (1 2) in cell B9 form a naked pair
    /// B ··· ·5· ··· | ··· ··· ··· | 4·· ·5· ···      with pseudo-cell AB8, and so (1 2)
    ///   ··· ··9 ··· | ··· ··· ··· | 7·· ··9 ···      may be removed from cells A9 and B7
    /// ```
    fn check_type_three(&self, board: &Board, effects: &mut Effects) -> bool {
        if !(2..=4).contains(&self.roof_extras.len()) {
            return false;
        }

        let mut action = Action::new(if self.roof_has_both {
            Strategy::UniqueRectangle
        } else {
            Strategy::AlmostUniqueRectangle
        });
        self.add_clues_for_all_corner_cells(&mut action);
        action.clue_cell_for_knowns(Verdict::Secondary, self.roof_left, self.roof_left_extras);
        action.clue_cell_for_knowns(Verdict::Secondary, self.roof_right, self.roof_right_extras);

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

                    let mut found = false;
                    for known in knowns {
                        let erase = cells & board.candidate_cells(known);
                        if !erase.is_empty() {
                            found = true;
                            action.erase_cells(erase, known)
                        }
                    }
                    if found {
                        for (cell, knowns) in peer_knowns_combo {
                            action.clue_cell_for_knowns(Verdict::Secondary, *cell, *knowns);
                        }
                    }
                    break;
                }
            }
        }

        // println!("type 3 {} - {}", self.rectangle, action);
        effects.add_action(action)
    }

    /// When one of the pair candidates is locked in the two non-bi-value neighbors,
    /// the other pair candidate may be removed from them.
    ///
    /// ```
    ///    1   2   3     4   5   6     7   8   9
    ///   ··· ··· ··· | ··· ··· ··· | ··· ··· ···
    /// A ··· ·5· ··· | ··· ··· ··· | ··· ·5· ···
    ///   ··· ··9 ··· | ··· ··· ··· | ··· ··9 ···
    ///               |             |
    ///   ··· ·2· ··· | ··· ··· ··· | ··· 1·· ···
    /// B ··· ·5· ··· | ·5· ·5· ··· | ··· 45· ···  ←-- 9 must appear in B2 or B8, so 5 may be removed
    ///   ··· ··9 ··· | ··· ··· ··· | ··· ··9 ···      from them to avoid the deadly rectangle
    /// ```
    fn check_type_four(&self, board: &Board, effects: &mut Effects) -> bool {
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

            let mut action = Action::new(if self.roof_has_both {
                Strategy::UniqueRectangle
            } else {
                Strategy::AlmostUniqueRectangle
            });
            action.erase_cells(self.roof & board.candidate_cells(erase), erase);
            action.clue_cells_for_knowns(Verdict::Primary, self.floor, self.pair);
            action.clue_cells_for_known(Verdict::Secondary, self.roof, required);

            // println!("type 4 {} - {}", self.rectangle, action);
            if effects.add_action(action) {
                return true;
            }
        }

        false
    }

    /// When two opposite corners contain only the pair and one of the pair must appear
    /// twice in the rectangle, the other of the pair may be removed from the first corners.
    ///
    /// This happens when either candidate is locked in both opposite sides in any houses.
    /// The left side could be locked in the column while the right is locked in the block, etc.
    ///
    /// ```
    ///    1   2   3     4   5   6     7   8   9
    ///   ··· ··· ··· | ··· ··· ··· | ··· ·2· ···
    /// A ··· ·5· ··· | ··· ··· ··· | ··· 45· ···
    ///   ··· ··9 ··9 | ··· ··9 ··· | ··· ··9 ···
    ///               |             |
    ///   ··· 1·· ··· | ··· ··· ··· | ··· ··· ···
    /// B ··· 45· ··· | ··· ··· ··· | ··· ·5· ···  ←-- 5 must appear in cells A2 or A8 and also in B2 or B8,
    ///   ··· ··9 ··· | ··· ··· ··9 | ··9 ··9 ···      and so 9 may be removed from cells A2 and B8
    /// ```
    fn check_type_five(&self, board: &Board, effects: &mut Effects) -> bool {
        let mut keep = None;

        let sides = vec![
            [
                (self.floor_left, self.roof_left),
                (self.floor_right, self.roof_right),
            ],
            [
                (self.floor_left, self.roof_right),
                (self.floor_right, self.roof_left),
            ],
        ];
        'found: for known in self.pair {
            for sides in &sides {
                if sides.iter().all(|(floor, roof)| {
                    floor
                        .common_houses(*roof)
                        .into_iter()
                        .any(|house| board.house_candidate_cells(house, known).len() == 2)
                }) {
                    keep = Some(known);
                    break 'found;
                }
            }
        }

        if let Some(keep) = keep {
            let erase = (self.pair - keep).as_single().unwrap();
            let mut action = Action::new(if self.roof_has_both {
                Strategy::UniqueRectangle
            } else {
                Strategy::AlmostUniqueRectangle
            });
            action.erase_cells(self.floor, erase);
            action.clue_cells_for_knowns(Verdict::Primary, self.roof, self.pair);
            action.clue_cells_for_known(Verdict::Primary, self.floor, keep);

            // println!("type 5 {} - {}", self.rectangle, action);
            effects.add_action(action)
        } else {
            false
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
    fn test_type_one() {
        let parser = Parse::wiki();
        let (board, effects, failed) = parser.parse(
            "k0k02109050h81031181110c21g1030k410sgkgs03418111gki8ish6g60hh009412181g40981h0h02105030h41g421410h03810911g4jkgkh4034109hgi0815048h8810h21h005032i0q810511g141282o",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_unique_rectangles(&board, true) {
            let mut action =
                Action::new_erase_knowns(Strategy::UniqueRectangle, cell!("D1"), knowns!("2 9"));
            action.clue_cells_for_knowns(Verdict::Primary, cells!("D9 F1 F9"), knowns!("2 9"));
            action.clue_cell_for_knowns(Verdict::Secondary, cell!("D1"), knowns!("1 5"));
            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }

    #[test]
    fn test_type_one_almost() {
        let parser = Parse::wiki();
        let (board, effects, failed) = parser.parse(
            "k0k02109050h81031181110c21g1030k410sgkgs03418111gki8ish2g60hh009412181g40981h0h02105030h41g421410h03810911g4jkgkh4034109hgi0815048h8810h21h005032i0q810511g141282o",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_unique_rectangles(&board, true) {
            let mut action =
                Action::new_erase(Strategy::AlmostUniqueRectangle, cell!("D1"), known!("9"));
            action.clue_cells_for_knowns(Verdict::Primary, cells!("D9 F1 F9"), knowns!("2 9"));
            action.clue_cell_for_knowns(Verdict::Secondary, cell!("D1"), knowns!("1 5"));
            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }

    #[test]
    fn test_type_two() {
        let parser = Parse::packed_with_options(Options::all());
        let (board, effects, failed) = parser.parse(
            "42.9..386 .6.2..794 8.9.6.251 7....3.25 9..1.26.3 2..5....8 ..4.2.567 6827..439 ......812",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_unique_rectangles(&board, true) {
            let mut action =
                Action::new_erase_cells(Strategy::UniqueRectangle, cells!("A3 C6"), known!("7"));
            action.clue_cells_for_knowns(Verdict::Primary, cells!("A5 A6 H5 H6"), knowns!("1 5"));
            action.clue_cells_for_known(Verdict::Secondary, cells!("A5 A6"), known!("7"));
            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }

    #[test]
    fn test_type_two_almost() {
        let parser = Parse::wiki();
        let (board, effects, failed) = parser.parse(
            "0h0552g150420981211a211a059a9241g10h8148g10o214g051103410ia2agog09g20511g11g9003cg05214g09050q2a11kgmgg24g810aga0h8805o21121412181054112120h09g118l8582ohojg810305",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_unique_rectangles(&board, true) {
            let mut action = Action::new_erase_cells(
                Strategy::AlmostUniqueRectangle,
                cells!("A3 C6"),
                known!("7"),
            );
            action.clue_cells_for_knowns(Verdict::Primary, cells!("H5 H6"), knowns!("1 5"));
            action.clue_cells_for_knowns(Verdict::Primary, cells!("A5"), knowns!("5"));
            action.clue_cells_for_knowns(Verdict::Primary, cells!("A6"), knowns!("1"));
            action.clue_cells_for_known(Verdict::Secondary, cells!("A5 A6"), known!("7"));
            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }

    #[test]
    fn test_type_two_diagonal() {
        let parser = Parse::wiki();
        let (board, effects, failed) = parser.parse(
            "814kg10s2u246c116e110922812m41i42mg42i4k621sg134812m6e05g10h215081030950418128g11c0334240h2803114c4c0h64g181gq4g055g81j0j822jagg1181032k09g441i4ga214a5454h40h81he",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_unique_rectangles(&board, true) {
            let mut action =
                Action::new_erase_cells(Strategy::UniqueRectangle, cells!("A9 C9 G7"), known!("6"));
            action.clue_cells_for_knowns(Verdict::Primary, cells!("B7 B9 H7 H9"), knowns!("2 9"));
            action.clue_cells_for_known(Verdict::Secondary, cells!("B7 H9"), known!("6"));
            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }

    #[test]
    fn test_type_three() {
        let parser = Parse::wiki();
        let (board, effects, failed) = parser.parse(
            "gg800541081102igiggi4111g2210408gg81g209200go2o00540100812g08104607g3i2i21040h10k002k08009811240g80gi8j0j20440g18a20928o1g050i05210agal2ko80hqgi100g8a05o2o8i0ia40",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_unique_rectangles(&board, true) {
            let mut action = Action::new(Strategy::UniqueRectangle);
            action.erase_knowns(cell!("H8"), knowns!("4 9"));
            action.erase_knowns(cell!("J8"), knowns!("6 9"));
            action.clue_cells_for_knowns(Verdict::Primary, cells!("D2 D8 F2 F8"), knowns!("1 5"));
            action.clue_cell_for_knowns(Verdict::Secondary, cell!("A8"), knowns!("4 6 9"));
            action.clue_cell_for_knowns(Verdict::Secondary, cell!("B8"), knowns!("4 9"));
            action.clue_cell_for_knowns(Verdict::Secondary, cell!("D8"), knowns!("4 6"));
            action.clue_cell_for_knowns(Verdict::Secondary, cell!("F8"), knowns!("6 9"));
            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }

    #[test]
    fn test_type_three_almost() {
        let parser = Parse::wiki();
        let (board, effects, failed) = parser.parse(
            "gg800541081102igiggi4111g2210408gg81g209200go2o00540100812g08104607g3g2i21040h10k002k08009811240g80gi8j0i20440g18a20928o1g050i05210agal2ko80hqgi100g8a05o2o8i0ia40",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_unique_rectangles(&board, true) {
            let mut action = Action::new(Strategy::AlmostUniqueRectangle);
            action.erase_knowns(cell!("H8"), knowns!("4 9"));
            action.erase_knowns(cell!("J8"), knowns!("6 9"));
            action.clue_cells_for_knowns(Verdict::Primary, cells!("D2 F2 F8"), knowns!("1"));
            action.clue_cells_for_knowns(Verdict::Primary, cells!("D2 D8 F2"), knowns!("5"));
            action.clue_cell_for_knowns(Verdict::Secondary, cell!("A8"), knowns!("4 6 9"));
            action.clue_cell_for_knowns(Verdict::Secondary, cell!("B8"), knowns!("4 9"));
            action.clue_cell_for_knowns(Verdict::Secondary, cell!("D8"), knowns!("4 6"));
            action.clue_cell_for_knowns(Verdict::Secondary, cell!("F8"), knowns!("6 9"));
            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }

    #[test]
    fn test_type_four() {
        let parser = Parse::wiki();
        let (board, effects, failed) = parser.parse(
            "k0k02109050h81031181110c21g1030k410sgkgs03418111gki8is12g60hh009412181g40981h0h02105030h41g421410h03810911g4jkgkh4034109hgi081l0k8h8810h21h005032i0q810511g141282o",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_unique_rectangles(&board, true) {
            let mut action = Action::new(Strategy::UniqueRectangle);
            action.erase_cells(cells!("H1 H2"), known!("9"));
            action.clue_cells_for_knowns(Verdict::Primary, cells!("A1 A2"), knowns!("7 9"));
            action.clue_cells_for_known(Verdict::Secondary, cells!("H1 H2"), known!("7"));
            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }

    #[test]
    fn test_type_four_almost() {
        let parser = Parse::wiki();
        let (board, effects, failed) = parser.parse(
            "k0k02109050h81031181110c21g1030k410sgkgs03418111gki8is12g60hh009412181g40981h0h02105030h41g421410h03810911g4jkgkh4034109hgi081l048h8810h21h005032i0q810511g141282o",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_unique_rectangles(&board, true) {
            let mut action = Action::new(Strategy::AlmostUniqueRectangle);
            action.erase(cell!("H1"), known!("9"));
            action.clue_cells_for_knowns(Verdict::Primary, cells!("A1 A2"), knowns!("7 9"));
            action.clue_cells_for_known(Verdict::Secondary, cells!("H1 H2"), known!("7"));
            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }

    #[test]
    fn test_type_five() {
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
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_unique_rectangles(&board, true) {
            let mut action =
                Action::new_erase_cells(Strategy::UniqueRectangle, cells!("E6 F1"), known!("2"));
            action.clue_cells_for_known(Verdict::Primary, cells!("E1 F6"), known!("2"));
            action.clue_cells_for_known(Verdict::Primary, cells!("E1 E6 F1 F6"), known!("8"));
            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }
}
