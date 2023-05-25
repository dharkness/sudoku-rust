use crate::layout::{Board, Cell, Coord, House, Known};

pub fn find_deadly_rectangles(board: &Board) -> bool {
    let mut found = false;

    for i in 0..2 {
        for (from, to) in BLOCKS[i] {
            let from = House::block(Coord::new(from));
            let to = House::block(Coord::new(to));

            for ((tl, bl), (tr, br)) in COORDS[i] {
                let tl = from.cell(Coord::new(tl));
                let br = to.cell(Coord::new(br));
                if !board.is_known(tl) || board.value(tl) != board.value(br) {
                    continue;
                }

                let bl = from.cell(Coord::new(bl));
                let tr = to.cell(Coord::new(tr));
                if !board.is_known(bl) || board.value(bl) != board.value(tr) {
                    continue;
                }

                println!("Deadly rectangle found: ({}, {}) ({}, {})", tl, bl, tr, br);
                found = true;
            }
        }
    }

    found
}

pub fn creates_deadly_rectangle(board: &Board, cell: Cell, known: Known) -> bool {
    let cell_coord = cell.coord_in_block().u8();
    let block = cell.block();
    let block_coord = block.coord().u8();
    let known_value = known.value();

    for i in 0..2 {
        for (f, t) in BLOCKS[i] {
            let mut from = House::block(Coord::new(f));
            let mut to = House::block(Coord::new(t));
            if f == block_coord {
                // use it
            } else if t == block_coord {
                (from, to) = (to, from);
            } else {
                continue;
            }

            for ((tl, bl), (tr, br)) in COORDS[i] {
                let mut top_left = from.cell(Coord::new(tl));
                let mut bottom_left = from.cell(Coord::new(bl));
                let mut top_right = to.cell(Coord::new(tr));
                let mut bottom_right = to.cell(Coord::new(br));
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
                if !board.is_known(bottom_left) || board.value(bottom_left) != board.value(top_right) {
                    continue;
                }

                // println!("Creates deadly rectangle: ({}, {}) ({}, {})", top_left, bottom_left, top_right, bottom_right);
                return true;
            }
        }
    }

    false
}

/// The block pairs (from, to) to check for deadly rectangles.
/// All possible rectangles between the two blocks are checked.
const BLOCKS: [[(u8, u8); 9]; 2] = [
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
const COORDS: [[((u8, u8), (u8, u8)); 27]; 2] = [
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
