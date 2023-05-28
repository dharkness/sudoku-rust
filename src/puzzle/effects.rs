use core::fmt;

use crate::layout::{Cell, Known};

use super::{Action, Board, Error, Strategy};

/// Collects actions and errors encountered while modifying a board.
#[derive(Clone, Debug)]
pub struct Effects {
    errors: Vec<Error>,
    actions: Vec<Action>,
}

impl Effects {
    pub fn new() -> Self {
        Self {
            errors: vec![],
            actions: vec![],
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn add_error(&mut self, error: Error) {
        self.errors.push(error);
    }

    pub fn has_actions(&self) -> bool {
        !self.actions.is_empty()
    }

    pub fn add_action(&mut self, action: Action) {
        self.actions.push(action);
    }

    pub fn add_set(&mut self, strategy: Strategy, cell: Cell, known: Known) {
        self.add_action(Action::new_set(strategy, cell, known));
    }

    pub fn add_erase(&mut self, strategy: Strategy, cell: Cell, known: Known) {
        self.add_action(Action::new_erase(strategy, cell, known));
    }

    pub fn apply(&self, board: &mut Board, effects: &mut Effects) {
        self.actions
            .iter()
            .for_each(|action| action.apply(board, effects));
    }

    pub fn apply_all(&self, board: &mut Board) -> Option<Effects> {
        let mut effects = self.clone();
        loop {
            if effects.has_errors() {
                return Some(effects);
            }
            if !effects.has_actions() {
                return None;
            }
            let mut next = Effects::new();
            effects.apply(board, &mut next);
            effects = next;
        }
    }

    pub fn print_errors(&self) {
        if self.has_errors() {
            println!("Errors:");
            for error in &self.errors {
                println!("- {}", error);
            }
        }
    }
}

impl fmt::Display for Effects {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.has_errors() {
            write!(f, "Errors:")?;
            for error in &self.errors {
                write!(f, "\n- {}", error)?;
            }
        }
        if self.has_actions() {
            if self.has_errors() {
                write!(f, "\n\n")?;
            }
            write!(f, "Actions:")?;
            for action in &self.actions {
                write!(f, "\n- {}", action)?;
            }
        }
        Ok(())
    }
}
