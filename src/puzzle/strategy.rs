use std::fmt;

/// Identifies the logic used to solve cells and remove candidates.
///
/// - Strategy stays a simple high-level enum with no values
/// - Rule specifies subtype or rule with knowns/cells/houses (see comments below)
///   - Strategy Intersection Removal has Line/Box Reduction and Pointing Pair/Triple
/// - Deduction combines the Strategy and Clue with Effects (sets and erases)
///
/// Add Class (groupings)?
/// - Naked Candidates
/// - Hidden Candidates
/// - Intersection Removal
/// - Fish
/// - ...kinda breaks down after that
///
/// What's the point? Want to be able to filter rules to apply (automatically),
/// and then really only peers and singles? This is a tool for creating and solving
/// puzzles automatically. The UI is just for fun and to learn Rust.
///
/// Add Difficulty? sudokuwiki.org only has four:
/// - Basic
/// - Tough
/// - Diabolical
/// - Extreme
///
/// What is the purpose of this project?
/// - learn Rust
/// - have fun
/// - exercise my brain
/// - Create a generalized solver using inference chains
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum Strategy {
    // these become the Clues; copy and generalize for Strategy
    /// The player or parser has provided a given (clue).
    Given, // (Known, Cell)
    /// The player has solved a cell.
    Solve, // (Known, Cell)
    /// The player has erased a candidate from a cell.
    Erase, // (Known, Cell)

    /// When a cell becomes solved, the value may be removed as a candidate
    /// from every cell in the same row, column or box.
    Peer, // (Known, Cell)

    /// A candidate that may only appear in two cells in one segment of a block
    /// may be removed from the other two segments in the segment's row or column.
    ///
    /// This is one form of intersection removals.
    PointingPair, // (Known, block House, House, (Cell, Cell))
    /// A candidate that may only appear in three cells one segment of a block
    /// may be removed from the other two segments in the segment's row or column.
    ///
    /// This is one form of intersection removals.
    PointingTriple, // (Known, block House, House, (Cell, Cell, Cell))
    /// A candidate that may only appear in one segment of a block
    /// may be removed from the other cells in the block.
    ///
    /// This is one form of intersection removals.
    BoxLineReduction, // (Known, block House, House)

    /// A cell with one candidate remaining may be solved.
    NakedSingle, // (Known, Cell)
    /// A candidate that may only appear in one cell in a house may be solved.
    HiddenSingle, // (Known, House, Cell)

    /// Two cells in a house and with the same two candidates remaining
    /// may remove those candidates from all other cells in that house.
    NakedPair, // (KnownSet, House, CoordSet)
    /// Two candidates remaining in two cells in a house
    /// may remove all other candidates in those cells.
    HiddenPair, // (KnownSet, House, CoordSet)

    /// Three cells in a house and with the same three candidates remaining
    /// may remove those candidates from all other cells in that house.
    NakedTriple, // (KnownSet, House, CoordSet)
    /// Three candidates remaining in three cells in a house
    /// may remove all other candidates in those cells.
    HiddenTriple, // (KnownSet, House, CoordSet)

    /// Four cells in a house and with the same four candidates remaining
    /// may remove those candidates from all other cells in that house.
    NakedQuad, // (KnownSet, House, CoordSet)
    /// Four candidates remaining in four cells in a house
    /// may remove all other candidates in those cells.
    HiddenQuad, // (KnownSet, House, CoordSet)

    XWing,     // (Known, mains HouseSet, crosses HouseSet)
    Swordfish, // (Known, mains HouseSet, crosses HouseSet)
    Jellyfish, // (Known, mains HouseSet, crosses HouseSet)

    Bug,                // (Cell, Cell, Cell)
    AvoidableRectangle, // (CellSet) - all unsolved cells
    SinglesChain,       // (Known, Vec<Cell>)
    Skyscraper,         // (Known, floor (Cell, Cell), ceiling (Cell, Cell))
    YWing,              // (Known, pivot Cell, arms (Cell, Cell))
    XYZWing,            // (Known, pivot Cell, arms (Cell, Cell))

    XYChain,         // (Known, Vec<Cell>)
    UniqueRectangle, // (Cell, Cell, Cell, Cell)

    EmptyRectangle, // (Known, Block, Row, Column, Cell) - CellSet instead of three houses

    BruteForce,
}

impl Strategy {
    pub fn label(&self) -> &str {
        match self {
            Self::Given => "Given",
            Self::Solve => "Solve",
            Self::Erase => "Erase",
            Self::Peer => "Peer",
            Self::PointingPair => "Pointing Pair",
            Self::PointingTriple => "Pointing Triple",
            Self::BoxLineReduction => "Box/Line Reduction",
            Self::NakedSingle => "Naked Single",
            Self::HiddenSingle => "Hidden Single",
            Self::NakedPair => "Naked Pair",
            Self::HiddenPair => "Hidden Pair",
            Self::NakedTriple => "Naked Triple",
            Self::HiddenTriple => "Hidden Triple",
            Self::NakedQuad => "Naked Quad",
            Self::HiddenQuad => "Hidden Quad",
            Self::XWing => "X-Wing",
            Self::Swordfish => "Swordfish",
            Self::Jellyfish => "Jellyfish",
            Self::Bug => "BUG",
            Self::AvoidableRectangle => "Avoidable Rectangle",
            Self::SinglesChain => "Singles Chain",
            Self::Skyscraper => "Skyscraper",
            Self::YWing => "Y-Wing",
            Self::XYZWing => "XYZ-Wing",
            Self::XYChain => "XY-Chain",
            Self::UniqueRectangle => "Unique Rectangle",
            Self::EmptyRectangle => "Empty Rectangle",
            Self::BruteForce => "Brute Force",
        }
    }
}

impl fmt::Display for Strategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}
