use std::collections::HashMap;
use std::time::Duration;

use crate::layout::{Cell, Known};
use crate::puzzle::{Action, Board, Effects, Strategy};
use crate::solve::Difficulty;

/// One of these methods is called for each puzzle run through the solver.
pub trait Reporter {
    /// The givens for a puzzle create an invalid puzzle.
    fn invalid(
        &self,
        givens: &str,
        start: &Board,
        errors: &Effects,
        cell: Cell,
        known: Known,
        runtime: Duration,
    );

    /// One of the solver techniques produced an invalid puzzle.
    #[allow(clippy::too_many_arguments)]
    fn failed(
        &self,
        givens: &str,
        start: &Board,
        stopped: &Board,
        action: &Action,
        errors: &Effects,
        runtime: Duration,
        counts: &HashMap<Strategy, i32>,
    );

    /// The puzzle could not be solved using the given techniques.
    fn unsolved(
        &self,
        givens: &str,
        start: &Board,
        stopped: &Board,
        runtime: Duration,
        counts: &HashMap<Strategy, i32>,
    );

    /// The puzzle was fully solved.
    fn solved(
        &self,
        givens: &str,
        solution: &Board,
        difficulty: Difficulty,
        runtime: Duration,
        counts: &HashMap<Strategy, i32>,
    );
}
