use crate::puzzle::{Board, Difficulty, Effects, Strategy};

use super::algorithms;

/// Names and categorizes a solver technique.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Technique {
    strategy: Strategy,
    difficulty: Difficulty,
    label: &'static str,
    solve: TechniqueFunc,
}

impl Technique {
    pub const fn new(strategy: Strategy, solve: TechniqueFunc) -> Technique {
        Technique {
            strategy,
            difficulty: strategy.difficulty(),
            label: strategy.label(),
            solve,
        }
    }

    pub const fn strategy(&self) -> Strategy {
        self.strategy
    }

    pub const fn difficulty(&self) -> Difficulty {
        self.difficulty
    }

    pub const fn label(&self) -> &'static str {
        self.label
    }

    pub fn solve(&self, board: &Board, single: bool) -> Option<Effects> {
        (self.solve)(board, single)
    }
}

type TechniqueFunc = fn(board: &Board, single: bool) -> Option<Effects>;

/// All techniques implemented by this solver.
#[rustfmt::skip]
pub const TECHNIQUES: [Technique; 28] = [
    Technique::new(Strategy::Peer, algorithms::find_peers),
    Technique::new(Strategy::NakedSingle, algorithms::find_naked_singles),
    Technique::new(Strategy::HiddenSingle, algorithms::find_hidden_singles),

    Technique::new(Strategy::NakedPair, algorithms::find_naked_pairs),
    Technique::new(Strategy::NakedTriple, algorithms::find_naked_triples),
    Technique::new(Strategy::NakedQuad, algorithms::find_naked_quads),
    Technique::new(Strategy::HiddenPair, algorithms::find_hidden_pairs),
    Technique::new(Strategy::HiddenTriple, algorithms::find_hidden_triples),
    Technique::new(Strategy::HiddenQuad, algorithms::find_hidden_quads),
    Technique::new(Strategy::IntersectionRemoval, algorithms::find_intersection_removals),

    Technique::new(Strategy::XWing, algorithms::find_x_wings),
    Technique::new(Strategy::TwoStringKite, algorithms::find_two_string_kites),
    Technique::new(Strategy::SinglesChain, algorithms::find_singles_chains),
    Technique::new(Strategy::YWing, algorithms::find_y_wings),
    Technique::new(Strategy::EmptyRectangle, algorithms::find_empty_rectangles),
    Technique::new(Strategy::Swordfish, algorithms::find_swordfish),
    Technique::new(Strategy::XYZWing, algorithms::find_xyz_wings),
    Technique::new(Strategy::AvoidableRectangle, algorithms::find_avoidable_rectangles),

    Technique::new(Strategy::Jellyfish,algorithms::find_jellyfish),
    Technique::new(Strategy::Skyscraper,algorithms::find_skyscrapers),
    Technique::new(Strategy::XYChain, algorithms::find_xy_chains),
    Technique::new(Strategy::UniqueRectangle, algorithms::find_unique_rectangles),
    Technique::new(Strategy::AlmostUniqueRectangle, algorithms::find_almost_unique_rectangles),
    Technique::new(Strategy::Fireworks,algorithms::find_fireworks),
    Technique::new(Strategy::ExtendedUniqueRectangle, algorithms::find_extended_unique_rectangles),
    Technique::new(Strategy::HiddenUniqueRectangle, algorithms::find_hidden_unique_rectangles),
    Technique::new(Strategy::WXYZWing, algorithms::find_wxyz_wings),

    // BUG causes unavoidable rectangles in several puzzles which UR fixes
    Technique::new(Strategy::Bug,algorithms::find_bugs),
];

/// All techniques except finding peers.
#[rustfmt::skip]
pub const NON_PEER_TECHNIQUES: [Technique; 27] = [
    TECHNIQUES[1],  TECHNIQUES[2],  TECHNIQUES[3],  TECHNIQUES[4],  TECHNIQUES[5],
    TECHNIQUES[6],  TECHNIQUES[7],  TECHNIQUES[8],  TECHNIQUES[9],  TECHNIQUES[10],
    TECHNIQUES[11], TECHNIQUES[12], TECHNIQUES[13], TECHNIQUES[14], TECHNIQUES[15],
    TECHNIQUES[16], TECHNIQUES[17], TECHNIQUES[18], TECHNIQUES[19], TECHNIQUES[20],
    TECHNIQUES[21], TECHNIQUES[22], TECHNIQUES[23], TECHNIQUES[24], TECHNIQUES[25],
    TECHNIQUES[26], TECHNIQUES[27],
];

/// All techniques that cannot be handled automatically by the [`Board`].
#[rustfmt::skip]
pub const MANUAL_TECHNIQUES: [Technique; 25] = [
    TECHNIQUES[3],  TECHNIQUES[4],  TECHNIQUES[5],  TECHNIQUES[6],  TECHNIQUES[7],
    TECHNIQUES[8],  TECHNIQUES[9],  TECHNIQUES[10], TECHNIQUES[11], TECHNIQUES[12],
    TECHNIQUES[13], TECHNIQUES[14], TECHNIQUES[15], TECHNIQUES[16], TECHNIQUES[17],
    TECHNIQUES[18], TECHNIQUES[19], TECHNIQUES[20], TECHNIQUES[21], TECHNIQUES[22],
    TECHNIQUES[23], TECHNIQUES[24], TECHNIQUES[25], TECHNIQUES[26], TECHNIQUES[27],
];
