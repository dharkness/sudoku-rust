use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::puzzle::{Board, Effects, Strategy};
use crate::solve::{Difficulty, TECHNIQUES};

pub enum Resolution {
    Solved(Board, Difficulty, Duration, HashMap<Strategy, i32>),
    Unsolved(Board, Duration, HashMap<Strategy, i32>),
    Failed(Board, Effects, Duration, HashMap<Strategy, i32>),
}

pub struct Solver {}

impl Solver {
    pub const fn new() -> Solver {
        Solver {}
    }

    pub fn solve(&self, start: &Board, effects: &Effects) -> Resolution {
        let runtime = Instant::now();
        let mut board = *start;
        let mut actions = effects.clone();
        let mut counts = HashMap::new();
        let mut difficulty = Difficulty::Basic;

        loop {
            if actions.has_actions() {
                loop {
                    if actions.has_errors() {
                        return Resolution::Failed(
                            board,
                            actions.clone(),
                            runtime.elapsed(),
                            counts,
                        );
                    }
                    if !actions.has_actions() {
                        break;
                    }
                    let mut next = Effects::new();
                    for action in actions.actions() {
                        if action.apply(&mut board, &mut next) {
                            let count = counts.entry(action.strategy()).or_default();
                            *count += 1;
                        }
                    }
                    actions = next;
                }
            }

            if board.is_solved() {
                return Resolution::Solved(board, difficulty, runtime.elapsed(), counts);
            }

            let mut found = false;
            for solver in TECHNIQUES {
                if let Some(moves) = solver.solve(&board) {
                    if solver.difficulty() > difficulty {
                        difficulty = solver.difficulty()
                    }
                    actions = moves;
                    found = true;
                    break;
                }
            }
            if !found {
                return Resolution::Unsolved(board, runtime.elapsed(), counts);
            }
        }
    }
}
