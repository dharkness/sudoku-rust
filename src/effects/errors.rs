use crate::layout::{Cell, House};

#[derive(Clone, Debug)]
pub enum Error {
    /// The unsolved cell has no more candidates remaining.
    UnsolvableCell(Cell),
    /// An unsolved value has no more candidate cells in the house.
    UnsolvableHouse(House),
    /// The house contains two solved cells with the same value. FIXME impossible
    Duplicate(House),
}
