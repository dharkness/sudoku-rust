use crate::layout::Board;

use super::{Error, Action, Actions};

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

    pub fn apply_all(&self, board: &mut Board, effects: &mut Effects) {
        self.actions.apply_all(board, effects);
    }
}
