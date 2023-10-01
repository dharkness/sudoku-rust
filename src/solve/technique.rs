use crate::puzzle::{Board, Effects};

use super::algorithms;

/// Names and categorizes a solver technique.
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
    Basic,
    Tough,
    Diabolical,
    Extreme,
}

type TechniqueFunc = fn(&Board) -> Option<Effects>;

#[rustfmt::skip]
pub const TECHNIQUES: [Technique; 19] = [
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
    Technique::new(Difficulty::Tough, "swordfish", algorithms::find_swordfish),
    Technique::new(Difficulty::Tough, "xyz-wing", algorithms::find_xyz_wings),

    Technique::new(Difficulty::Diabolical, "jellyfish", algorithms::find_jellyfish),
    Technique::new(Difficulty::Diabolical, "skyscraper", algorithms::find_skyscrapers),
    Technique::new(Difficulty::Diabolical, "avoidable rectangle", algorithms::find_avoidable_rectangles),
    Technique::new(Difficulty::Diabolical, "xy-chain", algorithms::find_xy_chains),
    Technique::new(Difficulty::Diabolical, "unique rectangle", algorithms::find_unique_rectangles),

    // kills some unique rectangles, breaking the puzzle
    Technique::new(Difficulty::Tough, "bug", algorithms::find_bugs),

    Technique::new(Difficulty::Extreme, "empty rectangle", algorithms::find_empty_rectangles),
];
