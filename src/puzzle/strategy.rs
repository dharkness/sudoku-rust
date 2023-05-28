/// Identifies the logic used to solve cells and remove candidates.
#[derive(Clone, Debug)]
pub enum Strategy {
    /// When a cell becomes solved, the value may be removed as a candidate
    /// from every cell in the same row, column or box.
    Neighbor,
    /// A cell with one candidate remaining may be solved.
    NakedSingle,
    /// A candidate that may only appear in one cell in a house may be solved.
    HiddenSingle,
    /// A candidate that may only appear in two cells in one segment of a block
    /// may be removed from the other two segments in the segment's row or column.
    ///
    /// This is one form of intersection removals.
    PointingPair,
    /// A candidate that may only appear in three cells one segment of a block
    /// may be removed from the other two segments in the segment's row or column.
    ///
    /// This is one form of intersection removals.
    PointingTriple,
    /// A candidate that may only appear in one segment of a block
    /// may be removed from the other cells in the block.
    ///
    /// This is one form of intersection removals.
    BoxLineReduction,
}
