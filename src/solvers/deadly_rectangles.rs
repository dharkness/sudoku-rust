use crate::layout::{Cell, Coord, House, Known, Rectangle};
use crate::puzzle::Board;

/// Finds all existing deadly rectangles in the board.
///
/// A deadly rectangle occurs when two cells in one block
/// and two cells in another block form a rectangle
/// where the same values appears in opposite corners.
/// This is not allowed because the two values could be swapped,
/// and every valid Sudoku solution must be unique.
///
/// # Example
///
/// ```text
///   123456789
/// A ·········
/// B ·2··3····  ←-- not allowed since the 2s and 3s may be swapped;
/// C ·3··2····      the left pair are in block 1, the right pair in block 2
/// D ·········
/// E ···7···4·  ←-- allowed since the 7s and 4s may not be swapped;
/// F ·········      no two adjacent corners are in the same block
/// G ·········
/// H ···4···7·
/// J ·········
/// ```
pub fn find_deadly_rectangles(board: &Board) -> Option<Vec<Rectangle>> {
    let mut found: Vec<Rectangle> = vec![];

    for horiz_vert in 0..2 {
        for (from, to) in BLOCKS[horiz_vert] {
            for ((tl, bl), (tr, br)) in CELL_COORDS[horiz_vert] {
                let top_left = from.cell(tl);
                let bottom_right = to.cell(br);
                if !board.is_known(top_left) || board.value(top_left) != board.value(bottom_right) {
                    continue;
                }

                let bottom_left = from.cell(bl);
                let top_right = to.cell(tr);
                if !board.is_known(bottom_left)
                    || board.value(bottom_left) != board.value(top_right)
                {
                    continue;
                }

                found.push(Rectangle::from(
                    top_left,
                    top_right,
                    bottom_right,
                    bottom_left,
                ));
            }
        }
    }

    if found.is_empty() {
        None
    } else {
        Some(found)
    }
}

/// Finds all deadly rectangles that would be formed if the given cell were set to the given value.
pub fn creates_deadly_rectangles(
    board: &Board,
    cell: Cell,
    known: Known,
) -> Option<Vec<Rectangle>> {
    let mut found: Vec<Rectangle> = vec![];

    let cell_coord = cell.coord_in_block();
    let block = cell.block();
    let known_value = known.value();

    for horiz_vert in 0..2 {
        for (mut from, mut to) in BLOCKS[horiz_vert] {
            if from == block {
                // use it
            } else if to == block {
                (from, to) = (to, from);
            } else {
                continue;
            }

            for ((tl, bl), (tr, br)) in CELL_COORDS[horiz_vert] {
                let mut top_left = from.cell(tl);
                let mut bottom_left = from.cell(bl);
                let mut top_right = to.cell(tr);
                let mut bottom_right = to.cell(br);
                if tl == cell_coord {
                    // use it
                } else if bl == cell_coord {
                    (top_left, bottom_left) = (bottom_left, top_left);
                    (top_right, bottom_right) = (bottom_right, top_right);
                } else {
                    continue;
                }

                if known_value != board.value(bottom_right) {
                    continue;
                }
                if !board.is_known(bottom_left)
                    || board.value(bottom_left) != board.value(top_right)
                {
                    continue;
                }

                found.push(Rectangle::from(
                    top_left,
                    top_right,
                    bottom_right,
                    bottom_left,
                ));
            }
        }
    }

    if found.is_empty() {
        None
    } else {
        Some(found)
    }
}

/// A pair of coordinates, either two different boxes or two different cells in the same box.
type IndexPair = (u8, u8);

/// A pair of cell coordinates in a box.
type CoordPair = (Coord, Coord);

/// The block pairs (from, to) to check for deadly rectangles.
/// All possible rectangles between the two blocks are checked
/// using the coordinates below.
const BLOCKS: [[(House, House); 9]; 2] = {
    #[rustfmt::skip]
    const BLOCKS: [[IndexPair; 9]; 2] = [
        // horizontal
        [
            (0, 1), (0, 2), (1, 2),
            (3, 4), (3, 5), (4, 5),
            (6, 7), (6, 8), (7, 8),
        ],
        // vertical
        [
            (0, 3), (0, 6), (3, 6),
            (1, 4), (1, 7), (4, 7),
            (2, 5), (2, 8), (5, 8),
        ],
    ];
    const DEFAULT: House = House::block(Coord::new(0));

    let mut blocks: [[(House, House); 9]; 2] = [[(DEFAULT, DEFAULT); 9]; 2];
    let mut horiz_vert = 0;

    while horiz_vert < 2 {
        let mut i = 0;
        while i < 9 {
            let (f, t) = BLOCKS[horiz_vert][i];
            blocks[horiz_vert][i] = (House::block(Coord::new(f)), House::block(Coord::new(t)));
            i += 1;
        }
        horiz_vert += 1;
    }

    blocks
};

/// Cell coordinates (top-left, bottom-right) for each rectangle.
/// each in a different block in the pairs above.
const CELL_COORDS: [[(CoordPair, CoordPair); 27]; 2] = {
    #[rustfmt::skip]
    const COORDS: [[(IndexPair, IndexPair); 27]; 2] = [
        // horizontal
        [
            ((0, 3), (0, 3)), ((0, 3), (1, 4)), ((0, 3), (2, 5)),
            ((0, 6), (0, 6)), ((0, 6), (1, 7)), ((0, 6), (2, 8)),

            ((1, 4), (0, 3)), ((1, 4), (1, 4)), ((1, 4), (2, 5)),
            ((1, 7), (0, 6)), ((1, 7), (1, 7)), ((1, 7), (2, 8)),

            ((2, 5), (0, 3)), ((2, 5), (1, 4)), ((2, 5), (2, 5)),
            ((2, 8), (0, 6)), ((2, 8), (1, 7)), ((2, 8), (2, 8)),

            ((3, 6), (3, 6)), ((3, 6), (4, 7)), ((3, 6), (5, 8)),

            ((4, 7), (3, 6)), ((4, 7), (4, 7)), ((4, 7), (5, 8)),

            ((5, 8), (3, 6)), ((5, 8), (4, 7)), ((5, 8), (5, 8)),
        ],
        // vertical
        [
            ((0, 1), (0, 1)), ((0, 1), (3, 4)), ((0, 1), (6, 7)),
            ((0, 2), (0, 2)), ((0, 2), (3, 5)), ((0, 2), (6, 8)),

            ((3, 4), (0, 1)), ((3, 4), (3, 4)), ((3, 4), (6, 7)),
            ((3, 5), (0, 2)), ((3, 5), (3, 5)), ((3, 5), (6, 8)),

            ((6, 7), (0, 1)), ((6, 7), (3, 4)), ((6, 7), (6, 7)),
            ((6, 8), (0, 2)), ((6, 8), (3, 5)), ((6, 8), (6, 8)),

            ((1, 2), (1, 2)), ((1, 2), (4, 5)), ((1, 2), (7, 8)),

            ((4, 5), (1, 2)), ((4, 5), (4, 5)), ((4, 5), (7, 8)),

            ((7, 8), (1, 2)), ((7, 8), (4, 5)), ((7, 8), (7, 8)),
        ],
    ];
    const DEFAULT_COORD: Coord = Coord::new(0);
    const DEFAULT: (CoordPair, CoordPair) = (
        (DEFAULT_COORD, DEFAULT_COORD),
        (DEFAULT_COORD, DEFAULT_COORD),
    );

    let mut coords: [[(CoordPair, CoordPair); 27]; 2] = [[DEFAULT; 27]; 2];
    let mut horiz_vert = 0;

    while horiz_vert < 2 {
        let mut i = 0;
        while i < 27 {
            let ((tl, bl), (tr, br)) = COORDS[horiz_vert][i];
            coords[horiz_vert][i] = (
                (Coord::new(tl), Coord::new(bl)),
                (Coord::new(tr), Coord::new(br)),
            );
            i += 1;
        }
        horiz_vert += 1;
    }

    coords
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::knowns::known::known;
    use crate::puzzle::Effects;

    #[test]
    fn find() {
        for horiz_vert in 0..2 {
            for (from, to) in BLOCKS[horiz_vert] {
                for ((tl, bl), (tr, br)) in CELL_COORDS[horiz_vert] {
                    let tl = from.cell(tl.into());
                    let br = to.cell(br.into());
                    let bl = from.cell(bl.into());
                    let tr = to.cell(tr.into());

                    let mut board = Board::new();
                    let mut effects = Effects::new();

                    board.set_known(tl, known!(1), &mut effects);
                    board.set_known(tr, known!(2), &mut effects);
                    board.set_known(br, known!(1), &mut effects);
                    board.set_known(bl, known!(2), &mut effects);

                    let found = find_deadly_rectangles(&board);
                    let want = Rectangle::new(tl, br);

                    assert!(found.is_some(), "deadly rectangle {} not found", want);
                    let found = found.unwrap();
                    assert_eq!(1, found.len(), "wrong count for {}", want);
                    assert_eq!(want, found[0]);
                }
            }
        }
    }

    #[test]
    fn creates() {
        const KNOWNS: [Known; 4] = [known!(1), known!(2), known!(1), known!(2)];

        fn test(cells: [Cell; 4]) {
            let want = Rectangle::new(cells[0], cells[2]);

            for i in 0..4 {
                let mut board = Board::new();
                let mut effects = Effects::new();

                for j in 0..4 {
                    if i != j {
                        board.set_known(cells[j], KNOWNS[j], &mut effects);
                    }
                }

                let found = creates_deadly_rectangles(&board, cells[i], KNOWNS[i]);

                assert!(found.is_some(), "deadly rectangle {} not found", want);
                let found = found.unwrap();
                assert_eq!(1, found.len(), "wrong count for {}", want);
                assert_eq!(want, found[0]);
            }
        }

        for horiz_vert in 0..2 {
            for (from, to) in BLOCKS[horiz_vert] {
                for ((tl, bl), (tr, br)) in CELL_COORDS[horiz_vert] {
                    test([
                        from.cell(tl.into()),
                        to.cell(tr.into()),
                        to.cell(br.into()),
                        from.cell(bl.into()),
                    ]);
                }
            }
        }
    }
}
