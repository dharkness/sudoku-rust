use crate::layout::{Cell, Known, Rectangle};
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
    let solved = board.solved();
    let found: Vec<Rectangle> = Rectangle::iter()
        .filter(|r| solved.has_all(r.cells))
        .filter(|r| board.value(r.top_left) == board.value(r.bottom_right))
        .filter(|r| board.value(r.top_right) == board.value(r.bottom_left))
        .collect();

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
    if !board.is_candidate(cell, known) || board.is_known(cell) {
        return None;
    }

    let value = known.value();
    let solved = board.solved();
    let found: Vec<Rectangle> = Rectangle::iter()
        .filter(|r| r.cells.has(cell))
        .filter(|r| (r.cells - solved).size() == 1)
        .map(|r| (r, r.with_origin(cell)))
        .filter(|(_, r)| board.value(r.bottom_right) == value)
        .filter(|(_, r)| board.value(r.top_right) == board.value(r.bottom_left))
        .map(|(r, _)| r)
        .collect();

    if found.is_empty() {
        None
    } else {
        Some(found)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::values::known::known;
    use crate::puzzle::Effects;

    #[test]
    fn find() {
        test_find(false);
    }

    #[test]
    fn find_ignores_givens() {
        test_find(true);
    }

    fn test_find(givens: bool) {
        for rectangle in Rectangle::iter() {
            let tl = rectangle.top_left;
            let tr = rectangle.top_right;
            let bl = rectangle.bottom_left;
            let br = rectangle.bottom_right;

            let mut board = Board::new();
            let mut effects = Effects::new();

            board.set_known(tl, known!(1), &mut effects);
            if givens {
                board.set_given(tr, known!(2), &mut effects);
            } else {
                board.set_known(tr, known!(2), &mut effects);
            }
            board.set_known(br, known!(1), &mut effects);
            board.set_known(bl, known!(2), &mut effects);

            let found = find_deadly_rectangles(&board);
            if givens {
                assert!(found.is_none());
            } else {
                let want = Rectangle::new(tl, br);

                assert!(found.is_some(), "deadly rectangle {} not found", want);
                let found = found.unwrap();
                assert_eq!(1, found.len(), "wrong count for {}", want);
                assert_eq!(want, found[0]);
            }
        }
    }

    #[test]
    fn creates() {
        test_creates(false);
    }

    #[test]
    fn creates_ignores_givens() {
        test_creates(true);
    }

    fn test_creates(givens: bool) {
        const KNOWNS: [Known; 4] = [known!(1), known!(2), known!(1), known!(2)];

        fn test(givens: bool, rectangle: Rectangle) {
            let cells = [
                rectangle.top_left,
                rectangle.top_right,
                rectangle.bottom_right,
                rectangle.bottom_left,
            ];
            let want = Rectangle::new(cells[0], cells[2]);

            for i in 0..4 {
                let mut board = Board::new();
                let mut effects = Effects::new();
                let mut first = givens;

                for j in 0..4 {
                    if i != j {
                        if first {
                            board.set_given(cells[j], KNOWNS[j], &mut effects);
                            first = false;
                        } else {
                            board.set_known(cells[j], KNOWNS[j], &mut effects);
                        }
                    }
                }

                let found = creates_deadly_rectangles(&board, cells[i], KNOWNS[i]);
                if givens {
                    assert!(found.is_none());
                } else {
                    assert!(found.is_some(), "deadly rectangle {} not found", want);
                    let found = found.unwrap();
                    assert_eq!(1, found.len(), "wrong count for {}", want);
                    assert_eq!(want, found[0]);
                }
            }
        }

        for rectangle in Rectangle::iter() {
            test(givens, rectangle);
        }
    }
}
