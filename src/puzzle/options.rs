use crate::puzzle::Strategy;

/// Available options for working with a [`Board`].
///
/// The mutators return a copy of the options with the given option set
/// without affecting the original, and they can be chained for convenience.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Options {
    /// True stops applying automatic moves when an error is encountered.
    pub stop_on_error: bool,

    /// True removes candidates from peers when a cell is solved
    /// instead of adding actions to the given effects.
    pub remove_peers: bool,

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
            remove_peers: false,
            solve_naked_singles: false,
            solve_hidden_singles: false,
            solve_intersection_removals: false,
        }
    }

    pub const fn errors_and_peers() -> Self {
        Self {
            stop_on_error: true,
            remove_peers: true,
            solve_naked_singles: false,
            solve_hidden_singles: false,
            solve_intersection_removals: false,
        }
    }

    pub const fn all() -> Self {
        Self {
            stop_on_error: true,
            remove_peers: true,
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

    pub fn remove_peers(mut self) -> Self {
        self.remove_peers = true;
        self
    }

    pub fn return_peers(mut self) -> Self {
        self.remove_peers = false;
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
            Strategy::Peer => self.remove_peers,
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
    fn test_remove_peers_does_not_alter_original() {
        let options = Options::none();
        let copy = options.remove_peers();

        assert!(!options.remove_peers);
        assert!(copy.remove_peers);
    }

    #[test]
    fn test_return_peers() {
        let options = Options::none().remove_peers().return_peers();

        assert!(!options.remove_peers);
    }

    #[test]
    fn test_solve_singles() {
        let options = Options::none().solve_singles();

        assert!(options.solve_naked_singles);
        assert!(options.solve_hidden_singles);
    }

    #[test]
    fn test_should_apply() {
        let mut options = Options::none();

        assert_eq!(false, options.should_apply(Strategy::Peer));
        assert_eq!(false, options.should_apply(Strategy::NakedSingle));
        assert_eq!(false, options.should_apply(Strategy::HiddenSingle));
        assert_eq!(false, options.should_apply(Strategy::PointingPair));
        assert_eq!(false, options.should_apply(Strategy::PointingTriple));
        assert_eq!(false, options.should_apply(Strategy::BoxLineReduction));
        assert_eq!(false, options.should_apply(Strategy::Bug));

        options = options.remove_peers();
        assert_eq!(true, options.should_apply(Strategy::Peer));
        assert_eq!(false, options.should_apply(Strategy::NakedSingle));
        assert_eq!(false, options.should_apply(Strategy::HiddenSingle));
        assert_eq!(false, options.should_apply(Strategy::PointingPair));
        assert_eq!(false, options.should_apply(Strategy::PointingTriple));
        assert_eq!(false, options.should_apply(Strategy::BoxLineReduction));
        assert_eq!(false, options.should_apply(Strategy::Bug));

        options = options.solve_naked_singles();
        assert_eq!(true, options.should_apply(Strategy::Peer));
        assert_eq!(true, options.should_apply(Strategy::NakedSingle));
        assert_eq!(false, options.should_apply(Strategy::HiddenSingle));
        assert_eq!(false, options.should_apply(Strategy::PointingPair));
        assert_eq!(false, options.should_apply(Strategy::PointingTriple));
        assert_eq!(false, options.should_apply(Strategy::BoxLineReduction));
        assert_eq!(false, options.should_apply(Strategy::Bug));

        options = options.solve_hidden_singles();
        assert_eq!(true, options.should_apply(Strategy::Peer));
        assert_eq!(true, options.should_apply(Strategy::NakedSingle));
        assert_eq!(true, options.should_apply(Strategy::HiddenSingle));
        assert_eq!(false, options.should_apply(Strategy::PointingPair));
        assert_eq!(false, options.should_apply(Strategy::PointingTriple));
        assert_eq!(false, options.should_apply(Strategy::BoxLineReduction));
        assert_eq!(false, options.should_apply(Strategy::Bug));

        options = options.return_peers();
        assert_eq!(false, options.should_apply(Strategy::Peer));
        assert_eq!(true, options.should_apply(Strategy::NakedSingle));
        assert_eq!(true, options.should_apply(Strategy::HiddenSingle));
        assert_eq!(false, options.should_apply(Strategy::PointingPair));
        assert_eq!(false, options.should_apply(Strategy::PointingTriple));
        assert_eq!(false, options.should_apply(Strategy::BoxLineReduction));
        assert_eq!(false, options.should_apply(Strategy::Bug));

        options = options.return_singles();
        assert_eq!(false, options.should_apply(Strategy::Peer));
        assert_eq!(false, options.should_apply(Strategy::NakedSingle));
        assert_eq!(false, options.should_apply(Strategy::HiddenSingle));
        assert_eq!(false, options.should_apply(Strategy::PointingPair));
        assert_eq!(false, options.should_apply(Strategy::PointingTriple));
        assert_eq!(false, options.should_apply(Strategy::BoxLineReduction));
        assert_eq!(false, options.should_apply(Strategy::Bug));

        options = options.solve_intersection_removals();
        assert_eq!(false, options.should_apply(Strategy::Peer));
        assert_eq!(false, options.should_apply(Strategy::NakedSingle));
        assert_eq!(false, options.should_apply(Strategy::HiddenSingle));
        assert_eq!(true, options.should_apply(Strategy::PointingPair));
        assert_eq!(true, options.should_apply(Strategy::PointingTriple));
        assert_eq!(true, options.should_apply(Strategy::BoxLineReduction));
        assert_eq!(false, options.should_apply(Strategy::Bug));

        options = options.return_intersection_removals();
        assert_eq!(false, options.should_apply(Strategy::Peer));
        assert_eq!(false, options.should_apply(Strategy::NakedSingle));
        assert_eq!(false, options.should_apply(Strategy::HiddenSingle));
        assert_eq!(false, options.should_apply(Strategy::PointingPair));
        assert_eq!(false, options.should_apply(Strategy::PointingTriple));
        assert_eq!(false, options.should_apply(Strategy::BoxLineReduction));
        assert_eq!(false, options.should_apply(Strategy::Bug));
    }
}
