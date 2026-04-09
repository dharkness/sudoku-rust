use core::fmt;
use std::collections::HashMap;

use crate::layout::{Cell, CellSet, Digit, DigitSet};

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

    pub fn add_set(&mut self, strategy: Strategy, cell: Cell, digit: Digit) {
        self.add_action(Action::new_set(strategy, cell, digit));
    }

    pub fn sets(&self, cell: Cell, digit: Digit) -> bool {
        self.actions.iter().any(|action| action.sets(cell, digit))
    }

    pub fn add_erase(&mut self, strategy: Strategy, cell: Cell, digit: Digit) {
        self.add_action(Action::new_erase(strategy, cell, digit));
    }

    pub fn add_erase_cells(&mut self, strategy: Strategy, cells: CellSet, digit: Digit) {
        self.add_action(Action::new_erase_cells(strategy, cells, digit));
    }

    pub fn add_erase_digits(&mut self, strategy: Strategy, cell: Cell, digits: DigitSet) {
        self.add_action(Action::new_erase_digits(strategy, cell, digits));
    }

    pub fn erases(&self, cell: Cell, digit: Digit) -> bool {
        self.actions.iter().any(|action| action.erases(cell, digit))
    }

    pub fn erases_from_cells(&self, digit: Digit) -> CellSet {
        self.actions.iter().fold(CellSet::empty(), |acc, action| {
            acc | action.erases_from_cells(digit)
        })
    }

    pub fn erases_digits_from(&self, cell: Cell) -> DigitSet {
        self.actions.iter().fold(DigitSet::empty(), |acc, action| {
            acc | action.erases_digits_from(cell)
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

    pub fn affecting_digit(&self, digit: Digit) -> Self {
        let mut effects = Self::new();
        for action in self.actions.iter() {
            if action.affects_digit(digit) {
                effects.add_action(action.clone());
            }
        }
        effects
    }

    pub fn pop_action(&mut self) -> Option<Action> {
        self.actions.pop()
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

/// Asserts that an [`Action`] or [`Effects`] sets a [`Cell`] to a given [`Digit`].
///
/// Compile-time convenience that parses the cell and digit tokens and panics on invalid input.
/// For runtime parsing with error handling, use [`Cell::from_str`] and [`Digit::from_str`].
///
/// # Examples
///
/// ```
/// use sudoku_rust::{assert_set, layout::{Cell, Digit}, Action, Effects};
///
/// let action = Action::new_set(Strategy::Place, cell!(A2), digit!(5));
/// assert_set!(action, A2, 5);
///
/// let mut effects = Effects::new();
/// effects.add_set(Strategy::Place, cell!(B7), digit!(9));
/// assert_set!(effects, B7, 9);
/// ```
///
/// # Panics
///
/// Panics if:
/// - The provided cell or digit tokens are invalid (see [`Cell::from_str`] and [`Digit::from_str`]).
/// - The assertion fails because the value does not set the cell to the digit.
#[macro_export]
macro_rules! assert_set {
    ($value:expr, $cell:tt, $digit:tt $(,)?) => {{
        let __cell = cell!($cell);
        let __digit = digit!($digit);

        assert!(
            $value.sets(__cell, __digit),
            "expected {} to set {} to {}",
            stringify!($value),
            __cell,
            __digit
        );
    }};
}

/// Asserts that an [`Action`] or [`Effects`] erases a [`Digit`] from a [`Cell`].
///
/// Compile-time convenience that parses the cell and digit tokens and panics on invalid input.
/// For runtime parsing with error handling, use [`Cell::from_str`] and [`Digit::from_str`].
///
/// # Examples
///
/// ```
/// use sudoku_rust::{assert_erase, layout::{Cell, Digit}, Action, Effects};
///
/// let action = Action::new_erase(Strategy::Erase, cell!(A2), digit!(5));
/// assert_erase!(action, A2, 5);
///
/// let mut effects = Effects::new();
/// effects.add_erase(Strategy::Erase, cell!(B7), digit!(9));
/// assert_erase!(effects, B7, 9);
/// ```
///
/// # Panics
///
/// Panics if:
/// - The provided cell or digit tokens are invalid (see [`Cell::from_str`] and [`Digit::from_str`]).
/// - The assertion fails because the value does not erase the digit from the cell.
#[macro_export]
macro_rules! assert_erase {
    ($value:expr, $cell:tt, $digit:tt $(,)?) => {{
        let __cell = cell!($cell);
        let __digit = digit!($digit);

        assert!(
            $value.erases(__cell, __digit),
            "expected {} to erase {} from {}",
            stringify!($value),
            __digit,
            __cell
        );
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn new_and_errors_work() {
        let mut effects = Effects::new();

        assert!(effects.is_empty());
        assert!(!effects.has_errors());
        assert_eq!(0, effects.error_count());

        let error = Error::UnsolvableCell(cell!(A1));
        effects.add_error(error);

        assert!(effects.has_errors());
        assert_eq!(1, effects.error_count());
        assert_eq!(Some(&error), effects.errors().first());
        assert_eq!(1, effects.errors_iter().count());

        effects.clear_errors();
        assert!(!effects.has_errors());
    }

    #[test]
    fn action_helpers_and_counts() {
        let mut effects = Effects::new();

        assert!(!effects.add_action(Action::new(Strategy::Place)));

        effects.add_set(Strategy::Place, cell!(A1), digit!(1));
        effects.add_erase(Strategy::Erase, cell!(A1), digit!(2));
        effects.add_erase_cells(Strategy::Erase, cells![A1 B2], digit!(3));
        effects.add_erase_digits(Strategy::Erase, cell!(A1), digits![4 5]);

        assert!(effects.has_actions());
        assert_eq!(4, effects.action_count());

        let counts = effects.action_counts();
        assert_eq!(1, counts[&Strategy::Place]);
        assert_eq!(3, counts[&Strategy::Erase]);
    }

    #[test]
    fn erases_helpers_work() {
        let mut effects = Effects::new();
        effects.add_erase(Strategy::Erase, cell!(A1), digit!(2));
        effects.add_erase_digits(Strategy::Erase, cell!(A1), digits![4 5]);
        effects.add_erase_cells(Strategy::Erase, cells![A1], digit!(7));

        assert!(effects.erases(cell!(A1), digit!(2)));
        assert_eq!(cells![A1], effects.erases_from_cells(digit!(7)));
        assert_eq!(digits![2 4 5 7], effects.erases_digits_from(cell!(A1)));
    }

    #[test]
    fn affecting_filters_actions() {
        let mut effects = Effects::new();
        effects.add_set(Strategy::Place, cell!(A1), digit!(1));
        effects.add_erase(Strategy::Erase, cell!(B2), digit!(1));
        effects.add_erase(Strategy::Erase, cell!(C3), digit!(2));

        let by_cell = effects.affecting_cell(cell!(A1));
        assert_eq!(1, by_cell.action_count());

        let by_digit = effects.affecting_digit(digit!(1));
        assert_eq!(2, by_digit.action_count());
    }

    #[test]
    fn pop_without_and_take_actions() {
        let mut effects = Effects::new();
        effects.add_set(Strategy::Place, cell!(A1), digit!(1));
        effects.add_set(Strategy::Place, cell!(B2), digit!(2));

        let popped = effects.pop_action().unwrap();
        assert!(popped.sets(cell!(B2), digit!(2)));
        assert_eq!(1, effects.action_count());

        let filtered = effects.without_action(0);
        assert_eq!(0, filtered.action_count());

        let mut other = Effects::new();
        other.add_set(Strategy::Place, cell!(C3), digit!(3));
        effects.take_actions(other);

        assert_eq!(2, effects.action_count());
    }

    #[test]
    fn apply_updates_board() {
        let mut effects = Effects::new();
        effects.add_set(Strategy::Place, cell!(A1), digit!(1));

        let mut board = Board::new();
        let mut next = Effects::new();
        let change = effects.apply(&mut board, &mut next);

        assert_eq!(Change::Valid, change);
        assert_eq!(Some(digit!(1)), board.value(cell!(A1)).digit());
        assert!(next.is_empty());
    }

    #[test]
    fn apply_strategy_filters_actions() {
        let mut effects = Effects::new();
        effects.add_set(Strategy::Place, cell!(A1), digit!(1));
        effects.add_erase(Strategy::Erase, cell!(B2), digit!(2));

        let mut board = Board::new();
        let mut next = Effects::new();
        let change = effects.apply_strategy(&mut board, Strategy::Place, &mut next);

        assert_eq!(Change::Valid, change);
        assert_eq!(Some(digit!(1)), board.value(cell!(A1)).digit());
        assert!(board.is_candidate(cell!(B2), digit!(2)));
        assert!(next.is_empty());
    }

    #[test]
    fn apply_all_returns_errors() {
        let mut effects = Effects::new();
        effects.add_error(Error::UnsolvableCell(cell!(A1)));

        let mut board = Board::new();
        let result = effects.apply_all(&mut board);

        assert!(result.is_some());
        assert!(result.unwrap().has_errors());
    }

    #[test]
    fn from_action_creates_effects() {
        let action = Action::new_set(Strategy::Place, cell!(A1), digit!(1));
        let effects: Effects = action.into();

        assert!(effects.has_actions());
        assert_eq!(1, effects.action_count());
    }

    #[test]
    fn display_includes_errors_and_actions() {
        let mut effects = Effects::new();
        effects.add_error(Error::UnsolvableCell(cell!(A1)));
        effects.add_set(Strategy::Place, cell!(A1), digit!(1));

        let text = format!("{}", effects);

        assert!(text.contains("Errors:"));
        assert!(text.contains("Actions:"));
        assert!(text.contains("A1"));
    }
}
