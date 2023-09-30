use crate::puzzle::{Board, Effects};

/// Names and categorizes a solver function.
pub struct Solver {
    difficulty: Difficulty,
    name: &'static str,
    solve: SolverFunc,
}

impl Solver {
    pub const fn new(difficulty: Difficulty, name: &'static str, solve: SolverFunc) -> Solver {
        Solver {
            difficulty,
            name,
            solve,
        }
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub fn solve(&self, board: &Board) -> Option<Effects> {
        (self.solve)(board)
    }
}

/// Groups solvers by difficulty based on the SudokuWiki website.
pub enum Difficulty {
    Basic,
    Tough,
    Diabolical,
    Extreme,
}

type SolverFunc = fn(&Board) -> Option<Effects>;

#[rustfmt::skip]
pub const SOLVERS: [Solver; 19] = [
    Solver::new(Difficulty::Basic, "naked pair", super::naked_tuples::find_naked_pairs),
    Solver::new(Difficulty::Basic, "naked triple", super::naked_tuples::find_naked_triples),
    Solver::new(Difficulty::Basic, "naked quad", super::naked_tuples::find_naked_quads),
    Solver::new(Difficulty::Basic, "hidden pair", super::hidden_tuples::find_hidden_pairs),
    Solver::new(Difficulty::Basic, "hidden triple", super::hidden_tuples::find_hidden_triples),
    Solver::new(Difficulty::Basic, "hidden quad", super::hidden_tuples::find_hidden_quads),
    Solver::new(Difficulty::Basic, "intersection removal", super::intersection_removals::find_intersection_removals),

    Solver::new(Difficulty::Tough, "x-wing", super::fish::find_x_wings),
    Solver::new(Difficulty::Tough, "singles chain", super::singles_chains::find_singles_chains),
    Solver::new(Difficulty::Tough, "y-wing", super::y_wings::find_y_wings),
    Solver::new(Difficulty::Tough, "swordfish", super::fish::find_swordfish),
    Solver::new(Difficulty::Tough, "xyz-wing", super::xyz_wings::find_xyz_wings),
    Solver::new(Difficulty::Tough, "bug", super::bugs::find_bugs),

    Solver::new(Difficulty::Diabolical, "jellyfish", super::fish::find_jellyfish),
    Solver::new(Difficulty::Diabolical, "skyscraper", super::skyscrapers::find_skyscrapers),
    Solver::new(Difficulty::Diabolical, "avoidable rectangle", super::avoidable_rectangles::find_avoidable_rectangles),
    Solver::new(Difficulty::Diabolical, "xy-chain", super::xy_chains::find_xy_chains),
    Solver::new(Difficulty::Diabolical, "unique rectangle", super::unique_rectangles::find_unique_rectangles),

    Solver::new(Difficulty::Extreme, "empty rectangle", super::empty_rectangles::find_empty_rectangles),
];
