use crate::layout::{Cell, Known};
use crate::puzzle::{Board, Effects, Error, Strategy};

pub struct Parser {
    stop_on_error: bool,
    apply_deductions: bool,
}

impl Parser {
    pub fn new(stop_on_error: bool, apply_deductions: bool) -> Self {
        Parser {
            stop_on_error,
            apply_deductions,
        }
    }

    pub fn parse(&self, input: &str) -> (Board, Effects, Option<(Cell, Known)>) {
        let mut board = Board::new();
        let mut effects = Effects::new();
        let mut c = 0;

        for char in input.chars() {
            match char {
                ' ' | '\r' | '\n' | '|' | '_' => continue,
                '1'..='9' => {
                    let cell = Cell::new(c);
                    let known = Known::from(char);
                    let current = board.value(cell);
                    if current == known.value() {
                        continue;
                    }

                    if board.is_candidate(cell, known) {
                        board.set_known(cell, known, &mut effects);
                        if effects.has_errors() && self.stop_on_error {
                            return (board, effects, Some((cell, known)));
                        }
                        if self.apply_deductions {
                            effects.apply_all(&mut board);
                        } else {
                            effects.apply_all_strategy(&mut board, Strategy::Neighbor);
                        }
                        effects.clear_actions();
                    } else if self.stop_on_error {
                        if current.is_known() {
                            effects.add_error(Error::AlreadySolved(cell, known, current.known()));
                        } else {
                            effects.add_error(Error::NotCandidate(cell, known));
                        }
                        return (board, effects, Some((cell, known)));
                    }
                }
                _ => (),
            }

            c += 1;
        }

        (board, effects, None)
    }
}

/// Builds a new [`Board`] using the input string to set some cells,
/// and returns it along with any [`Action`]s and [`Error`]s that arise.
///
/// - Use a digit (1 to 9) to set a cell's value.
/// - Use whitespace, pipes, and underscores for readability.
/// - Use any other character to leave a cell unsolved.
///
/// This is designed for parsing valid puzzles. Attempting to set a cell
/// to a value that is proven not to be a candidate by the various built-in
/// solvers will be silently ignored.
pub fn parse_puzzle(input: &str) -> (Board, Effects) {
    let mut board = Board::new();
    let mut effects = Effects::new();
    let mut c = 0;

    for char in input.chars() {
        match char {
            ' ' | '\r' | '\n' | '|' | '_' => continue,
            '1'..='9' => {
                let cell: Cell = Cell::new(c);
                let known: Known = Known::from(char);
                if board.is_candidate(cell, known) {
                    board.set_known(cell, known, &mut effects);
                }
            }
            _ => (),
        }

        c += 1;
    }

    (board, effects)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let (mut board, effects) = parse_puzzle(
            "
            .1..7....
            2...4....
            .7.3.59..
            .29...4.5
            1..4.....
            ...9....2
            6..8.....
            952....1.
            ....6..7.
        ",
        );
        assert!(effects.apply_all(&mut board).is_none());

        let (mut want, effects) = parse_puzzle(
            "
            51.279.4.
            29.1465.7
            476385921
            .2961.4.5
            16542.79.
            .8495.162
            637891254
            952734.1.
            841562379
        ",
        );
        assert!(!effects.has_errors());

        assert!(effects.apply_all(&mut want).is_none());
        assert_eq!(want, board)
    }
}
