use crate::layout::{Board, Cell, Known};

use super::{Action, Actions, Error, Strategy};

#[derive(Clone, Debug)]
pub struct Effects {
    errors: Vec<Error>,
    actions: Actions,
}

impl Effects {
    pub fn new() -> Self {
        Self {
            errors: vec![],
            actions: Actions::new(),
        }
    }

    pub fn errors(&self) -> &[Error] {
        &self.errors
    }

    pub fn moves(&self) -> &Actions {
        &self.actions
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
        self.actions.add(action);
    }

    pub fn add_set(&mut self, strategy: Strategy, cell: Cell, known: Known) {
        self.add_action(Action::new_set(strategy, cell, known));
    }

    pub fn add_erase(&mut self, strategy: Strategy, cell: Cell, known: Known) {
        self.add_action(Action::new_erase(strategy, cell, known));
    }

    pub fn apply(&self, board: &mut Board, effects: &mut Effects) {
        self.actions.apply_all(board, effects);
    }

    pub fn apply_all(&self, board: &mut Board) -> bool {
        let mut effects = self.clone();
        loop {
            if effects.has_errors() {
                // print_candidates(&board);
                // println!("set known effects caused errors {:?}", effects.errors());
                return false;
            }
            if !effects.has_actions() {
                return true;
            }
            let mut next = Effects::new();
            effects.apply(board, &mut next);
            effects = next;
        }
    }
}
