use core::fmt;
use std::collections::HashMap;

use crate::layout::{Cell, CellSet, Known, KnownSet};

use super::{Action, Board, Change, Error, Strategy};

/// Collects actions and errors encountered while modifying a board.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Effects {
    errors: Vec<Error>,
    actions: Vec<Action>,
}

pub type Result = std::result::Result<Effects, Effects>;

impl Effects {
    pub const fn new() -> Self {
        Self {
            errors: vec![],
            actions: vec![],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty() && self.actions.is_empty()
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn clear_errors(&mut self) {
        self.errors = vec![];
    }

    pub fn errors(&self) -> &Vec<Error> {
        &self.errors
    }

    pub fn errors_iter(&self) -> impl Iterator<Item = &'_ Error> {
        self.errors.iter()
    }

    pub fn add_error(&mut self, error: Error) {
        self.errors.push(error);
    }

    pub fn print_errors(&self) {
        self.errors.iter().for_each(|error| println!("- {}", error));
    }

    pub fn has_actions(&self) -> bool {
        !self.actions.is_empty()
    }

    pub fn action_count(&self) -> usize {
        self.actions.len()
    }

    pub fn action_counts(&self) -> HashMap<Strategy, i32> {
        self.actions
            .iter()
            .fold(HashMap::new(), |mut counts, action| {
                let count = counts.entry(action.strategy()).or_default();
                *count += 1;
                counts
            })
    }

    pub fn clear_actions(&mut self) {
        self.actions = vec![];
    }

    pub fn actions(&self) -> &Vec<Action> {
        &self.actions
    }

    pub fn add_action(&mut self, action: Action) -> bool {
        if action.is_empty() {
            false
        } else {
            self.actions.push(action);
            true
        }
    }

    pub fn add_set(&mut self, strategy: Strategy, cell: Cell, known: Known) {
        self.add_action(Action::new_set(strategy, cell, known));
    }

    pub fn add_erase(&mut self, strategy: Strategy, cell: Cell, known: Known) {
        self.add_action(Action::new_erase(strategy, cell, known));
    }

    pub fn add_erase_cells(&mut self, strategy: Strategy, cells: CellSet, known: Known) {
        self.add_action(Action::new_erase_cells(strategy, cells, known));
    }

    pub fn add_erase_knowns(&mut self, strategy: Strategy, cell: Cell, knowns: KnownSet) {
        self.add_action(Action::new_erase_knowns(strategy, cell, knowns));
    }

    pub fn erases(&self, cell: Cell, known: Known) -> bool {
        self.actions.iter().any(|action| action.erases(cell, known))
    }

    pub fn erases_from_cells(&self, known: Known) -> CellSet {
        self.actions.iter().fold(CellSet::empty(), |acc, action| {
            acc | action.erases_from_cells(known)
        })
    }

    pub fn erases_knowns_from(&self, cell: Cell) -> KnownSet {
        self.actions.iter().fold(KnownSet::empty(), |acc, action| {
            acc | action.erases_knowns_from(cell)
        })
    }

    pub fn affecting_cell(&self, cell: Cell) -> Self {
        let mut effects = Self::new();
        for action in self.actions.iter() {
            if action.affects_cell(cell) {
                effects.add_action(action.clone());
            }
        }
        effects
    }

    pub fn affecting_known(&self, known: Known) -> Self {
        let mut effects = Self::new();
        for action in self.actions.iter() {
            if action.affects_known(known) {
                effects.add_action(action.clone());
            }
        }
        effects
    }

    pub fn without_action(&self, index: usize) -> Self {
        let mut effects = self.clone();
        effects.actions.remove(index);
        effects
    }

    pub fn take_actions(&mut self, mut from: Effects) {
        self.actions.append(&mut from.actions);
    }

    pub fn apply(&self, board: &mut Board, effects: &mut Effects) -> Change {
        self.actions.iter().fold(Change::None, |change, action| {
            change & action.apply(board, effects)
        })
    }

    pub fn apply_strategy(
        &self,
        board: &mut Board,
        strategy: Strategy,
        effects: &mut Effects,
    ) -> Change {
        self.actions.iter().fold(Change::None, |change, action| {
            if action.has_strategy(strategy) {
                change & action.apply(board, effects)
            } else {
                change
            }
        })
    }

    pub fn apply_all(&self, board: &mut Board) -> Option<Effects> {
        if self.has_errors() {
            return Some(self.clone());
        }
        if self.has_actions() {
            let mut next = Effects::new();
            self.apply(board, &mut next);
            if next.has_errors() {
                return Some(next);
            }
        }
        None
    }

    pub fn apply_all_strategy(&self, board: &mut Board, strategy: Strategy) -> Option<Effects> {
        let mut effects = self.clone();
        loop {
            if effects.has_errors() {
                return Some(effects);
            }
            if !effects.has_actions() {
                return None;
            }
            let mut next = Effects::new();
            effects.apply_strategy(board, strategy, &mut next);
            effects = next;
        }
    }

    pub fn print_actions(&self) {
        self.actions
            .iter()
            .for_each(|action| println!("- {}", action));
    }
}

impl From<Action> for Effects {
    fn from(action: Action) -> Self {
        let mut effects = Self::new();
        effects.add_action(action);
        effects
    }
}

impl fmt::Display for Effects {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.has_errors() {
            write!(f, "Errors:")?;
            self.errors
                .iter()
                .try_for_each(|error| write!(f, "\n- {}", error))?;
        }
        if self.has_actions() {
            if self.has_errors() {
                write!(f, "\n\n")?;
            }
            write!(f, "Actions:")?;
            self.actions
                .iter()
                .try_for_each(|action| write!(f, "\n- {}", action))?;
        }
        Ok(())
    }
}
