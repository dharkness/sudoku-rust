use crate::layout::{Cell, CellSet, KnownSet};

/// Combines two or more peer cells into a unit that can be treated as a single cell
/// when looking for naked/hidden singles and tuples.
///
/// It may appear in a row, column or block or in a block along with a row or column.
/// In the latter case, it's probably better to create two pseudo cells.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PseudoCell {
    pub pseudo: Cell,
    pub cells: CellSet,
    pub knowns: KnownSet,
}

impl PseudoCell {
    pub fn new(cells: CellSet, knowns: KnownSet) -> PseudoCell {
        PseudoCell {
            pseudo: cells.first().unwrap(),
            cells,
            knowns,
        }
    }
}
