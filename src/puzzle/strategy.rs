use std::fmt;

/// Identifies the logic used to solve cells and remove candidates.
///
/// - Strategy stays a simple high-level enum with no values
/// - Rule specifies subtype or rule with digits/cells/houses (see comments below)
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
    Give, // (Digit, Cell)
    /// The player has solved a cell.
    Place, // (Digit, Cell)
    /// The player has erased a candidate from a cell.
    Erase, // (Digit, Cell)

    /// When a cell becomes solved, the value may be removed as a candidate
    /// from every cell in the same row, column or box.
    Peer, // (Digit, Cell)

    /// A cell with one candidate remaining may be solved.
    NakedSingle, // (Digit, Cell)
    /// A candidate that may only appear in one cell in a house may be solved.
    HiddenSingle, // (Digit, House, Cell)

    /// Two cells in a house and with the same two candidates remaining
    /// may remove those candidates from all other cells in that house.
    NakedPair, // (DigitSet, House, CoordSet)
    /// Two candidates remaining in two cells in a house
    /// may remove all other candidates in those cells.
    HiddenPair, // (DigitSet, House, CoordSet)

    /// Three cells in a house and with the same three candidates remaining
    /// may remove those candidates from all other cells in that house.
    NakedTriple, // (DigitSet, House, CoordSet)
    /// Three candidates remaining in three cells in a house
    /// may remove all other candidates in those cells.
    HiddenTriple, // (DigitSet, House, CoordSet)

    /// Four cells in a house and with the same four candidates remaining
    /// may remove those candidates from all other cells in that house.
    NakedQuad, // (DigitSet, House, CoordSet)
    /// Four candidates remaining in four cells in a house
    /// may remove all other candidates in those cells.
    HiddenQuad, // (DigitSet, House, CoordSet)

    /// This strategy produces pointing pairs and triples and box/line reductions.
    IntersectionRemoval,
    /// A candidate that may only appear in two cells in one segment of a block
    /// may be removed from the other two segments in the segment's row or column.
    PointingPair, // (Digit, block House, House, (Cell, Cell))
    /// A candidate that may only appear in three cells one segment of a block
    /// may be removed from the other two segments in the segment's row or column.
    PointingTriple, // (Digit, block House, House, (Cell, Cell, Cell))
    /// A candidate that may only appear in one segment of a block
    /// may be removed from the other cells in the block.
    BoxLineReduction, // (Digit, block House, House)

    XWing,     // (Digit, mains HouseSet, crosses HouseSet)
    Swordfish, // (Digit, mains HouseSet, crosses HouseSet)
    Jellyfish, // (Digit, mains HouseSet, crosses HouseSet)

    Bug,                       // (Cell, Cell, Cell)
    AvoidableRectangle,        // (CellSet) - all unsolved cells
    TwoStringKite,             // (Digit, Vec<Cell>)
    SinglesChain,              // (Digit, Vec<Cell>)
    XCycle,                    // (Digit, Vec<Cell>)
    ThreeDMedusa,              // (Digit, Vec<Cell>)
    Skyscraper,                // (Digit, floor (Cell, Cell), ceiling (Cell, Cell))
    YWing,                     // (Digit, pivot Cell, arms (Cell, Cell))
    WWing,                     // (Digit, cells (Cell, Cell), links (Cell, Cell))
    ChuteRemotePair,           // (DigitSet, Cell, Cell)
    XYZWing,                   // (Digit, pivot Cell, arms (Cell, Cell))
    WXYZWing,                  // (Digit, pivot Cell, arms (Cell, Cell, Cell))
    AlignedPairExclusion,      // (Cell, Cell)
    AlternatingInferenceChain, // (Digit, Vec<Cell>)

    XYChain,                 // (Digit, Vec<Cell>)
    UniqueRectangle,         // (DigitSet, Cell, Cell, Cell, Cell)
    AlmostUniqueRectangle,   // (DigitSet, Cell, Cell, Cell, Cell)
    Fireworks,               // (DigitSet, Cell, Cell, Cell)
    ExtendedUniqueRectangle, // (DigitSet, Cell, Cell, Cell, Cell, Cell, Cell)
    HiddenUniqueRectangle,   // (DigitSet, Cell, Cell, Cell, Cell)
    RectangleElimination,    // (Digit, CellSet)

    BruteForce,
}

impl Strategy {
    pub const fn difficulty(&self) -> Difficulty {
        match self {
            Self::Give => Difficulty::Trivial,
            Self::Place => Difficulty::Trivial,
            Self::Erase => Difficulty::Trivial,

            Self::Peer => Difficulty::Trivial,
            Self::NakedSingle => Difficulty::Trivial,
            Self::HiddenSingle => Difficulty::Trivial,

            Self::NakedPair => Difficulty::Basic,
            Self::HiddenPair => Difficulty::Basic,
            Self::NakedTriple => Difficulty::Basic,
            Self::HiddenTriple => Difficulty::Basic,
            Self::NakedQuad => Difficulty::Basic,
            Self::HiddenQuad => Difficulty::Basic,
            Self::IntersectionRemoval => Difficulty::Basic,
            Self::PointingPair => Difficulty::Basic,
            Self::PointingTriple => Difficulty::Basic,
            Self::BoxLineReduction => Difficulty::Basic,

            Self::XWing => Difficulty::Tough,
            Self::TwoStringKite => Difficulty::Tough,
            Self::ChuteRemotePair => Difficulty::Tough,
            Self::YWing => Difficulty::Tough,
            Self::WWing => Difficulty::Tough,
            Self::RectangleElimination => Difficulty::Tough,
            Self::SinglesChain => Difficulty::Tough,
            Self::Swordfish => Difficulty::Tough,
            Self::XYZWing => Difficulty::Tough,
            Self::AvoidableRectangle => Difficulty::Tough,
            Self::Bug => Difficulty::Tough,

            Self::XCycle => Difficulty::Diabolical,
            Self::Skyscraper => Difficulty::Diabolical,
            Self::XYChain => Difficulty::Diabolical,
            Self::ThreeDMedusa => Difficulty::Diabolical,
            Self::Jellyfish => Difficulty::Diabolical,
            Self::UniqueRectangle => Difficulty::Diabolical,
            Self::AlmostUniqueRectangle => Difficulty::Diabolical,
            Self::Fireworks => Difficulty::Diabolical,
            Self::ExtendedUniqueRectangle => Difficulty::Diabolical,
            Self::HiddenUniqueRectangle => Difficulty::Diabolical,
            Self::WXYZWing => Difficulty::Diabolical,
            Self::AlignedPairExclusion => Difficulty::Diabolical,

            Self::AlternatingInferenceChain => Difficulty::Extreme,

            Self::BruteForce => Difficulty::BruteForce,
        }
    }

    pub const fn label(&self) -> &'static str {
        match self {
            Self::AlignedPairExclusion => "Aligned Pair Exclusion",
            Self::AlmostUniqueRectangle => "Almost Unique Rectangle",
            Self::AlternatingInferenceChain => "Alternating Inference Chain",
            Self::AvoidableRectangle => "Avoidable Rectangle",
            Self::BoxLineReduction => "Box/Line Reduction",
            Self::BruteForce => "Brute Force",
            Self::Bug => "BUG",
            Self::ChuteRemotePair => "Chute Remote Pair",
            Self::Erase => "Erase",
            Self::ExtendedUniqueRectangle => "Extended Unique Rectangle",
            Self::Fireworks => "Fireworks",
            Self::Give => "Give",
            Self::HiddenPair => "Hidden Pair",
            Self::HiddenQuad => "Hidden Quad",
            Self::HiddenSingle => "Hidden Single",
            Self::HiddenTriple => "Hidden Triple",
            Self::HiddenUniqueRectangle => "Hidden Unique Rectangle",
            Self::IntersectionRemoval => "Intersection Removal",
            Self::Jellyfish => "Jellyfish",
            Self::NakedPair => "Naked Pair",
            Self::NakedQuad => "Naked Quad",
            Self::NakedSingle => "Naked Single",
            Self::NakedTriple => "Naked Triple",
            Self::Peer => "Peer",
            Self::Place => "Place",
            Self::PointingPair => "Pointing Pair",
            Self::PointingTriple => "Pointing Triple",
            Self::RectangleElimination => "Rectangle Elimination",
            Self::SinglesChain => "Singles Chain",
            Self::Skyscraper => "Skyscraper",
            Self::Swordfish => "Swordfish",
            Self::ThreeDMedusa => "3D Medusa",
            Self::TwoStringKite => "Two-String Kite",
            Self::UniqueRectangle => "Unique Rectangle",
            Self::WWing => "W-Wing",
            Self::WXYZWing => "WXYZ-Wing",
            Self::XCycle => "X-Cycle",
            Self::XWing => "X-Wing",
            Self::XYChain => "XY-Chain",
            Self::XYZWing => "XYZ-Wing",
            Self::YWing => "Y-Wing",
        }
    }

    pub const fn acronym(&self) -> &'static str {
        match self {
            Self::AlignedPairExclusion => "AP",
            Self::AlmostUniqueRectangle => "AU",
            Self::AlternatingInferenceChain => "IC",
            Self::AvoidableRectangle => "AR",
            Self::BoxLineReduction => "BL",
            Self::BruteForce => "BF",
            Self::Bug => "BG",
            Self::ChuteRemotePair => "CR",
            Self::Erase => " E",
            Self::ExtendedUniqueRectangle => "ER",
            Self::Fireworks => "FW",
            Self::Give => " G",
            Self::HiddenPair => "HP",
            Self::HiddenQuad => "HQ",
            Self::HiddenSingle => "HS",
            Self::HiddenTriple => "HT",
            Self::HiddenUniqueRectangle => "HU",
            Self::IntersectionRemoval => "IR",
            Self::Jellyfish => "JF",
            Self::NakedPair => "NP",
            Self::NakedQuad => "NQ",
            Self::NakedSingle => "NS",
            Self::NakedTriple => "NT",
            Self::Peer => " P",
            Self::Place => " S",
            Self::PointingPair => "PP",
            Self::PointingTriple => "PT",
            Self::RectangleElimination => "RE",
            Self::SinglesChain => "SC",
            Self::Skyscraper => "SK",
            Self::Swordfish => "SF",
            Self::ThreeDMedusa => "3D",
            Self::TwoStringKite => "TS",
            Self::UniqueRectangle => "UR",
            Self::WWing => "WW",
            Self::WXYZWing => "WZ",
            Self::XCycle => "XC",
            Self::XWing => "XW",
            Self::XYChain => "XY",
            Self::XYZWing => "XZ",
            Self::YWing => "YW",
        }
    }
}

impl fmt::Display for Strategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Groups solvers by difficulty based on the SudokuWiki website.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum Difficulty {
    Trivial,
    Basic,
    Tough,
    Diabolical,
    Extreme,
    BruteForce,
}
