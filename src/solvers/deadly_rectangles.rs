use crate::layout::{Cell, House, Known, Rectangle};
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
            let from = House::block(from.into());
            let to = House::block(to.into());

            for ((tl, bl), (tr, br)) in COORDS[horiz_vert] {
                let tl = from.cell(tl.into());
                let br = to.cell(br.into());
                if !board.is_known(tl) || board.value(tl) != board.value(br) {
                    continue;
                }

                let bl = from.cell(bl.into());
                let tr = to.cell(tr.into());
                if !board.is_known(bl) || board.value(bl) != board.value(tr) {
                    continue;
                }

                found.push(Rectangle::from([tl, tr, br, bl]));
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

    let cell_coord = cell.coord_in_block().u8();
    let block_coord = cell.block().coord().u8();
    let known_value = known.value();

    for horiz_vert in 0..2 {
        for (f, t) in BLOCKS[horiz_vert] {
            let mut from = House::block(f.into());
            let mut to = House::block(t.into());
            if f == block_coord {
                // use it
            } else if t == block_coord {
                (from, to) = (to, from);
            } else {
                continue;
            }

            for ((tl, bl), (tr, br)) in COORDS[horiz_vert] {
                let mut top_left = from.cell(tl.into());
                let mut bottom_left = from.cell(bl.into());
                let mut top_right = to.cell(tr.into());
                let mut bottom_right = to.cell(br.into());
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

                found.push(Rectangle::from([
                    top_left,
                    top_right,
                    bottom_right,
                    bottom_left,
                ]));
            }
        }
    }

    if found.is_empty() {
        None
    } else {
        Some(found)
    }
}

/// A pair of coordinates, either two boxes or two cells in one box.
type CoordPair = (u8, u8);

/// The block pairs (from, to) to check for deadly rectangles.
/// All possible rectangles between the two blocks are checked
/// using the coordinates below.
#[rustfmt::skip]
const BLOCKS: [[CoordPair; 9]; 2] = [
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

/// Block coordinates (top-left, bottom-right) for each rectangle.
/// each in a different block in the pairs above.
#[rustfmt::skip]
const COORDS: [[(CoordPair, CoordPair); 27]; 2] = [
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
