use std::collections::HashMap;
use crate::effects::effects::Effects;

use crate::layout::{Board, Cell, Known, KnownSet};

#[derive(Clone, Debug)]
pub enum Strategy {
    Neighbor,
    NakedSingle,
    HiddenSingle,
    PointingPair,
    PointingTriple,
    BoxLineReduction,
}

#[derive(Clone, Debug)]
pub struct Action {
    strategy: Strategy,
    set: HashMap<Cell, Known>,
    erase: HashMap<Cell, KnownSet>,
}

impl Action {
    pub fn new(strategy: Strategy) -> Self {
        Self {
            strategy,
            set: HashMap::new(),
            erase: HashMap::new(),
        }
    }

    pub fn new_set(strategy: Strategy, cell: Cell, known: Known) -> Self {
        Self {
            strategy,
            set: HashMap::from([(cell, known)]),
            erase: HashMap::new(),
        }
    }

    pub fn new_erase(strategy: Strategy, cell: Cell, known: Known) -> Self {
        Self {
            strategy,
            set: HashMap::new(),
            erase: HashMap::from([(cell, KnownSet::empty() + known)]),
        }
    }

    pub fn set(&mut self, cell: Cell, known: Known) {
        self.set.insert(cell, known);
    }

    pub fn erase(&mut self, cell: Cell, known: Known) {
        *self.erase.entry(cell).or_insert_with(KnownSet::empty) += known;
    }

    pub fn apply(&self, board: &mut Board, effects: &mut Effects) {
        for (cell, knowns) in &self.erase {
            for known in knowns.iter() {
                // println!("erase {} from {}", known, cell);
                board.remove_candidate(*cell, known, effects);
            }
        }
        for (cell, known) in &self.set {
            // println!("set {} to {}", cell, known);
            board.set_known(*cell, *known, effects);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Actions {
    moves: Vec<Action>,
}

impl Actions {
    pub fn new() -> Self {
        Self { moves: vec![] }
    }

    pub fn is_empty(&self) -> bool {
        self.moves.is_empty()
    }

    pub fn size(&self) -> usize {
        self.moves.len()
    }

    pub fn add(&mut self, action: Action) {
        self.moves.push(action);
    }

    pub fn apply_all(&self, board: &mut Board, effects: &mut Effects) {
        for action in &self.moves {
            action.apply(board, effects);
        }
    }
}
