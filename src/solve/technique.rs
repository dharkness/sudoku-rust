use crate::puzzle::{Board, Effects};

use super::algorithms;

/// Names and categorizes a solver technique.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Technique {
    difficulty: Difficulty,
    name: &'static str,
    solve: TechniqueFunc,
}

impl Technique {
    pub const fn new(
        difficulty: Difficulty,
        name: &'static str,
        solve: TechniqueFunc,
    ) -> Technique {
        Technique {
            difficulty,
            name,
            solve,
        }
    }

    pub const fn difficulty(&self) -> Difficulty {
        self.difficulty
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub fn solve(&self, board: &Board) -> Option<Effects> {
        (self.solve)(board)
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
}

type TechniqueFunc = fn(&Board) -> Option<Effects>;

/// All techniques implemented by this solver.
#[rustfmt::skip]
pub const TECHNIQUES: [Technique; 22] = [
    Technique::new(Difficulty::Trivial, "peer", algorithms::find_peers),
    Technique::new(Difficulty::Trivial, "naked single", algorithms::find_naked_singles),
    Technique::new(Difficulty::Trivial, "hidden single", algorithms::find_hidden_singles),

    Technique::new(Difficulty::Basic, "naked pair", algorithms::find_naked_pairs),
    Technique::new(Difficulty::Basic, "naked triple", algorithms::find_naked_triples),
    Technique::new(Difficulty::Basic, "naked quad", algorithms::find_naked_quads),
    Technique::new(Difficulty::Basic, "hidden pair", algorithms::find_hidden_pairs),
    Technique::new(Difficulty::Basic, "hidden triple", algorithms::find_hidden_triples),
    Technique::new(Difficulty::Basic, "hidden quad", algorithms::find_hidden_quads),
    Technique::new(Difficulty::Basic, "intersection removal", algorithms::find_intersection_removals),

    Technique::new(Difficulty::Tough, "x-wing", algorithms::find_x_wings),
    Technique::new(Difficulty::Tough, "singles chain", algorithms::find_singles_chains),
    Technique::new(Difficulty::Tough, "y-wing", algorithms::find_y_wings),
    Technique::new(Difficulty::Tough, "empty rectangle", algorithms::find_empty_rectangles),
    Technique::new(Difficulty::Tough, "swordfish", algorithms::find_swordfish),
    Technique::new(Difficulty::Tough, "xyz-wing", algorithms::find_xyz_wings),
    Technique::new(Difficulty::Tough, "bug", algorithms::find_bugs),

    Technique::new(Difficulty::Diabolical, "jellyfish", algorithms::find_jellyfish),
    Technique::new(Difficulty::Diabolical, "skyscraper", algorithms::find_skyscrapers),
    Technique::new(Difficulty::Diabolical, "avoidable rectangle", algorithms::find_avoidable_rectangles),
    Technique::new(Difficulty::Diabolical, "xy-chain", algorithms::find_xy_chains),
    Technique::new(Difficulty::Diabolical, "unique rectangle", algorithms::find_unique_rectangles),
];

/// All techniques except finding peers.
#[rustfmt::skip]
pub const NON_PEER_TECHNIQUES: [Technique; 21] = [
    TECHNIQUES[1],  TECHNIQUES[2],  TECHNIQUES[3],  TECHNIQUES[4],  TECHNIQUES[5],
    TECHNIQUES[6],  TECHNIQUES[7],  TECHNIQUES[8],  TECHNIQUES[9],  TECHNIQUES[10],
    TECHNIQUES[11], TECHNIQUES[12], TECHNIQUES[13], TECHNIQUES[14], TECHNIQUES[15],
    TECHNIQUES[16], TECHNIQUES[17], TECHNIQUES[18], TECHNIQUES[19], TECHNIQUES[20],
    TECHNIQUES[21],
];

/// All techniques that cannot be handled automatically by the [`Board`].
#[rustfmt::skip]
pub const MANUAL_TECHNIQUES: [Technique; 19] = [
    TECHNIQUES[3],  TECHNIQUES[4],  TECHNIQUES[5],  TECHNIQUES[6],  TECHNIQUES[7],
    TECHNIQUES[8],  TECHNIQUES[9],  TECHNIQUES[10], TECHNIQUES[11], TECHNIQUES[12],
    TECHNIQUES[13], TECHNIQUES[14], TECHNIQUES[15], TECHNIQUES[16], TECHNIQUES[17],
    TECHNIQUES[18], TECHNIQUES[19], TECHNIQUES[20], TECHNIQUES[21],
];
