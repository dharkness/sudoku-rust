use crate::puzzle::Strategy;

/// Available options for working with a [`Board`][`super::Board`].
///
/// The mutators return a copy of the options with the given option set
/// without affecting the original, and they can be chained for convenience.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Options {
    /// True stops applying automatic moves when an error is encountered.
    pub stop_on_error: bool,

    /// True solves cells which have only one candidate remaining
    /// instead of adding actions to the given effects.
    pub solve_naked_singles: bool,

    /// True solves cells which are the only remaining candidate in a house
    /// instead of adding actions to the given effects.
    pub solve_hidden_singles: bool,

    /// True removes candidates using the pointing pairs/triples
    /// and box/line reduction strategies.
    ///
    /// Since the board doesn't detect these automatically
    /// as it does in in the TypeScript solver, the solver
    /// must be run every time the queue of actions is depleted.
    pub solve_intersection_removals: bool,
}

impl Options {
    pub const fn none() -> Self {
        Self {
            stop_on_error: false,
            solve_naked_singles: false,
            solve_hidden_singles: false,
            solve_intersection_removals: false,
        }
    }

    pub const fn errors() -> Self {
        Self {
            stop_on_error: true,
            solve_naked_singles: false,
            solve_hidden_singles: false,
            solve_intersection_removals: false,
        }
    }

    pub const fn all() -> Self {
        Self {
            stop_on_error: true,
            solve_naked_singles: true,
            solve_hidden_singles: true,
            solve_intersection_removals: true,
        }
    }

    pub fn stop_on_error(mut self) -> Self {
        self.stop_on_error = true;
        self
    }

    pub fn ignore_errors(mut self) -> Self {
        self.stop_on_error = false;
        self
    }

    pub fn solve_naked_singles(mut self) -> Self {
        self.solve_naked_singles = true;
        self
    }

    pub fn return_naked_singles(mut self) -> Self {
        self.solve_naked_singles = false;
        self
    }

    pub fn solve_hidden_singles(mut self) -> Self {
        self.solve_hidden_singles = true;
        self
    }

    pub fn return_hidden_singles(mut self) -> Self {
        self.solve_hidden_singles = false;
        self
    }

    pub fn solve_singles(mut self) -> Self {
        self.solve_naked_singles = true;
        self.solve_hidden_singles = true;
        self
    }

    pub fn return_singles(mut self) -> Self {
        self.solve_naked_singles = false;
        self.solve_hidden_singles = false;
        self
    }

    pub fn solve_intersection_removals(mut self) -> Self {
        self.solve_intersection_removals = true;
        self
    }

    pub fn return_intersection_removals(mut self) -> Self {
        self.solve_intersection_removals = false;
        self
    }

    pub fn should_apply(&self, strategy: Strategy) -> bool {
        match strategy {
            Strategy::Peer => true,
            Strategy::NakedSingle => self.solve_naked_singles,
            Strategy::HiddenSingle => self.solve_hidden_singles,
            Strategy::PointingPair => self.solve_intersection_removals,
            Strategy::PointingTriple => self.solve_intersection_removals,
            Strategy::BoxLineReduction => self.solve_intersection_removals,
            Strategy::BruteForce => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn defaults_match_none() {
        let none = Options::none();
        let defaulted = Options::default();

        assert_eq!(none, defaulted);
        assert!(!none.stop_on_error);
        assert!(!none.solve_naked_singles);
        assert!(!none.solve_hidden_singles);
        assert!(!none.solve_intersection_removals);
    }

    #[test]
    fn errors_and_all_set_expected_flags() {
        let errors = Options::errors();
        let all = Options::all();

        assert!(errors.stop_on_error);
        assert!(!errors.solve_naked_singles);
        assert!(!errors.solve_hidden_singles);
        assert!(!errors.solve_intersection_removals);

        assert!(all.stop_on_error);
        assert!(all.solve_naked_singles);
        assert!(all.solve_hidden_singles);
        assert!(all.solve_intersection_removals);
    }

    #[test]
    fn mutators_toggle_flags() {
        let base = Options::none();
        let updated = base
            .stop_on_error()
            .solve_naked_singles()
            .solve_hidden_singles()
            .solve_intersection_removals();

        assert!(!base.stop_on_error);
        assert!(updated.stop_on_error);
        assert!(updated.solve_naked_singles);
        assert!(updated.solve_hidden_singles);
        assert!(updated.solve_intersection_removals);

        let reverted = updated
            .ignore_errors()
            .return_naked_singles()
            .return_hidden_singles()
            .return_intersection_removals();

        assert!(!reverted.stop_on_error);
        assert!(!reverted.solve_naked_singles);
        assert!(!reverted.solve_hidden_singles);
        assert!(!reverted.solve_intersection_removals);
    }

    #[test]
    fn singles_helpers_set_and_clear_both_flags() {
        let options = Options::none().solve_singles();

        assert!(options.solve_naked_singles);
        assert!(options.solve_hidden_singles);

        let cleared = options.return_singles();
        assert!(!cleared.solve_naked_singles);
        assert!(!cleared.solve_hidden_singles);
    }

    #[test]
    fn should_apply_respects_flags() {
        let none = Options::none();

        assert!(none.should_apply(Strategy::Peer));
        assert!(none.should_apply(Strategy::BruteForce));
        assert!(!none.should_apply(Strategy::NakedSingle));
        assert!(!none.should_apply(Strategy::HiddenSingle));
        assert!(!none.should_apply(Strategy::PointingPair));
        assert!(!none.should_apply(Strategy::PointingTriple));
        assert!(!none.should_apply(Strategy::BoxLineReduction));
        assert!(!none.should_apply(Strategy::NakedPair));

        let singles = Options::none().solve_singles();
        assert!(singles.should_apply(Strategy::NakedSingle));
        assert!(singles.should_apply(Strategy::HiddenSingle));
        assert!(!singles.should_apply(Strategy::PointingPair));

        let intersections = Options::none().solve_intersection_removals();
        assert!(intersections.should_apply(Strategy::PointingPair));
        assert!(intersections.should_apply(Strategy::PointingTriple));
        assert!(intersections.should_apply(Strategy::BoxLineReduction));
        assert!(!intersections.should_apply(Strategy::HiddenSingle));
    }
}
