//! Shared command processing for console and TUI play modes.

use std::collections::VecDeque;
use std::fmt;
use std::time::Instant;

use clap::Args;
use itertools::Itertools;

use crate::build::{Finder, Generator};
use crate::io::{
    add_single_value_labels, format_for_fancy_console, format_for_wiki, format_grid, format_packed,
    format_runtime, Cancelable, Parse, Parser, SUDOKUWIKI_URL,
};
use crate::layout::{Cell, CellSet, Digit, DigitSet};
use crate::puzzle::{Action, Board, ChangeResult, Changer, Effects, Options, Strategy};
use crate::solve::{find_brute_force, BruteForceResult, TECHNIQUES};
use crate::symbols::{MISSING, UNSOLVED};

use super::deduction_merge::take_actions_with_rules;

const MAXIMUM_SOLUTIONS: usize = 100;

#[derive(Debug, Args, Clone, Copy, Default)]
pub struct PlayOptionsArgs {
    /// Automatically solve naked singles
    #[clap(short, long)]
    naked: bool,

    /// Automatically solve hidden singles
    #[clap(short, long)]
    hidden: bool,

    /// Automatically solve naked or hidden singles (same as --naked --hidden)
    #[clap(short, long)]
    singles: bool,

    /// Automatically solve intersection removals
    #[clap(short, long)]
    intersection: bool,
}

impl PlayOptionsArgs {
    pub fn options(&self) -> Options {
        Options {
            stop_on_error: true,
            solve_naked_singles: self.naked || self.singles,
            solve_hidden_singles: self.hidden || self.singles,
            solve_intersection_removals: self.intersection,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewPuzzleInput {
    Cancel,
    Empty,
    Puzzle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrintTarget {
    Givens,
    Solved,
    Candidate(Digit),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressStage {
    Generate,
    Find,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindFilter {
    Cell(Cell),
    Digit(Digit),
    Strategy(Strategy),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayCommand {
    ToggleOptions(Option<String>),
    NewPuzzle {
        input: NewPuzzleInput,
        puzzle: Option<String>,
    },
    CreatePuzzle,
    Print(Option<PrintTarget>),
    Export(Option<char>),
    Wiki,
    Grid,
    SetGiven {
        cells: CellSet,
        digit: Digit,
    },
    Solve {
        cells: CellSet,
        digit: Digit,
    },
    Erase {
        cells: CellSet,
        digits: DigitSet,
    },
    Find(Option<FindFilter>),
    Highlight {
        index: usize,
    },
    Apply(Option<usize>),
    Verify,
    Bingo,
    Reset,
    Undo,
    Redo,
    Help,
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Overlay {
    Help(String),
    Board {
        title: &'static str,
        lines: Vec<String>,
    },
    Text {
        title: &'static str,
        lines: Vec<String>,
    },
}

#[derive(Debug, Default)]
pub struct PlayOutput {
    pub messages: Vec<String>,
    pub overlay: Option<Overlay>,
    pub board_changed: bool,
    pub show_board: bool,
    pub quit: bool,
}

impl PlayOutput {
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.messages.push(message.into());
        self
    }

    pub fn with_overlay(mut self, overlay: Overlay) -> Self {
        self.overlay = Some(overlay);
        self
    }

    pub fn with_show_board(mut self) -> Self {
        self.show_board = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayError {
    MissingArguments(&'static str),
    InvalidArguments(String),
    UnknownCommand(String),
}

impl fmt::Display for PlayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlayError::MissingArguments(msg) => write!(f, "{}", msg),
            PlayError::InvalidArguments(msg) => write!(f, "{}", msg),
            PlayError::UnknownCommand(cmd) => write!(f, "Unknown command: {}", cmd),
        }
    }
}

pub fn parse_command_line(line: &str) -> Result<PlayCommand, PlayError> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Err(PlayError::MissingArguments(""));
    }

    let input = trimmed.to_uppercase();
    let parts = input.split_whitespace().collect_vec();
    let command = parts[0];

    match command {
        "O" => {
            let option = if parts.len() >= 2 {
                Some(parts[1].to_string())
            } else {
                None
            };
            Ok(PlayCommand::ToggleOptions(option))
        }
        "N" => Ok(PlayCommand::NewPuzzle {
            input: NewPuzzleInput::Puzzle,
            puzzle: None,
        }),
        "C" => Ok(PlayCommand::CreatePuzzle),
        "P" => {
            let target = if parts.len() >= 2 {
                let c = parts[1].chars().next().unwrap_or(UNSOLVED);
                match c {
                    'G' => Some(PrintTarget::Givens),
                    'S' => Some(PrintTarget::Solved),
                    '1'..='9' => Some(PrintTarget::Candidate(
                        c.to_string().parse::<Digit>().map_err(|e| {
                            PlayError::InvalidArguments(format!("Bad input: {}", e))
                        })?,
                    )),
                    _ => {
                        return Err(PlayError::InvalidArguments(format!(
                            "Invalid candidate \"{}\"",
                            c
                        )))
                    }
                }
            } else {
                None
            };
            Ok(PlayCommand::Print(target))
        }
        "X" => {
            let ch = if parts.len() >= 2 {
                parts[1].chars().next()
            } else {
                None
            };
            Ok(PlayCommand::Export(ch))
        }
        "W" => Ok(PlayCommand::Wiki),
        "M" => Ok(PlayCommand::Grid),
        "G" => {
            if parts.len() != 3 {
                return Err(PlayError::MissingArguments("G <cells> <digit>"));
            }
            let cells = CellSet::try_from(parts[1])
                .map_err(|e| PlayError::InvalidArguments(format!("Bad input: {}", e)))?;
            let digit = parts[2]
                .parse::<Digit>()
                .map_err(|e| PlayError::InvalidArguments(format!("Bad input: {}", e)))?;
            Ok(PlayCommand::SetGiven { cells, digit })
        }
        "S" => {
            if parts.len() != 3 {
                return Err(PlayError::MissingArguments("S <cells> <digit>"));
            }
            let cells = CellSet::try_from(parts[1])
                .map_err(|e| PlayError::InvalidArguments(format!("Bad input: {}", e)))?;
            let digit = parts[2]
                .parse::<Digit>()
                .map_err(|e| PlayError::InvalidArguments(format!("Bad input: {}", e)))?;
            Ok(PlayCommand::Solve { cells, digit })
        }
        "E" => {
            if parts.len() != 3 {
                return Err(PlayError::MissingArguments("E <cells> <digits>"));
            }
            let cells = CellSet::try_from(parts[1])
                .map_err(|e| PlayError::InvalidArguments(format!("Bad input: {}", e)))?;
            let digits = DigitSet::try_from(parts[2])
                .map_err(|e| PlayError::InvalidArguments(format!("Bad input: {}", e)))?;
            Ok(PlayCommand::Erase { cells, digits })
        }
        "F" => {
            let filter = if parts.len() == 2 {
                let token = parts[1];
                let mut filter = None;
                if token.len() == 1 {
                    if let Ok(digit) = token.parse::<Digit>() {
                        filter = Some(FindFilter::Digit(digit));
                    }
                }
                if filter.is_none() && token.len() == 2 {
                    if let Ok(cell) = token.parse::<Cell>() {
                        filter = Some(FindFilter::Cell(cell));
                    }
                }
                if filter.is_none() {
                    if let Ok(strategy) = Strategy::try_from(token) {
                        filter = Some(FindFilter::Strategy(strategy));
                    } else {
                        return Err(PlayError::InvalidArguments(format!(
                            "Unknown strategy: {}",
                            token
                        )));
                    }
                }
                filter
            } else {
                None
            };
            Ok(PlayCommand::Find(filter))
        }
        "H" => {
            if parts.len() != 2 {
                return Err(PlayError::MissingArguments("H <num>"));
            }
            let index = parts[1].parse::<usize>().unwrap_or(0);
            Ok(PlayCommand::Highlight { index })
        }
        "A" => {
            let index = if parts.len() >= 2 {
                Some(parts[1].parse::<usize>().unwrap_or(0))
            } else {
                None
            };
            Ok(PlayCommand::Apply(index))
        }
        "V" => Ok(PlayCommand::Verify),
        "B" => Ok(PlayCommand::Bingo),
        "R" => Ok(PlayCommand::Reset),
        "Z" => Ok(PlayCommand::Undo),
        "Y" => Ok(PlayCommand::Redo),
        "?" => Ok(PlayCommand::Help),
        "Q" => Ok(PlayCommand::Quit),
        _ => Err(PlayError::UnknownCommand(command.to_string())),
    }
}

pub struct PlayState {
    pub changer: Changer,
    boards: VecDeque<Board>,
    index: usize,
    deductions: Option<Effects>,
    deductions_strategy: Option<Strategy>,
    highlight: Option<Action>,
    cancelable: Cancelable,
}

impl PlayState {
    pub fn new(options: Options, puzzle: Option<String>) -> (Self, PlayOutput) {
        let mut state = Self {
            changer: Changer::new(options),
            boards: VecDeque::new(),
            index: 0,
            deductions: None,
            deductions_strategy: None,
            highlight: None,
            cancelable: Cancelable::new(),
        };

        let mut output = PlayOutput::default();
        match puzzle {
            Some(clues) => {
                let normalized = clues.trim().replace(' ', "").replace(MISSING, ".");
                let parser: Box<dyn Parser> = if normalized.len() >= 160 {
                    Box::new(Parse::wiki())
                } else {
                    Box::new(Parse::packed_with_player(state.changer))
                };
                let (board, effects, failure) = parser.parse(&normalized);
                state.boards.push_back(board);
                state.index = 0;

                if let Some((cell, digit)) = failure {
                    output.show_board = true;
                    output.messages.push(format!(
                        "==> Setting {} to {} will cause errors",
                        cell, digit
                    ));
                    output.messages.extend(errors_to_lines(&effects));
                } else {
                    output.show_board = true;
                }
            }
            None => {
                // Defer initialization to the caller (play/tui) so they can
                // generate a puzzle or decide how to seed the state.
            }
        }

        (state, output)
    }

    pub fn current(&self) -> &Board {
        self.boards.get(self.index).expect("board history is empty")
    }

    pub fn highlight(&self) -> Option<&Action> {
        self.highlight.as_ref()
    }

    pub fn apply(&mut self, command: PlayCommand) -> PlayOutput {
        match command {
            PlayCommand::ToggleOptions(flags) => self.toggle_options(flags),
            PlayCommand::NewPuzzle { input, puzzle } => self.new_puzzle(input, puzzle),
            PlayCommand::CreatePuzzle => self.create_puzzle(),
            PlayCommand::Print(target) => self.print_board(target),
            PlayCommand::Export(ch) => self.export_board(ch),
            PlayCommand::Wiki => self.print_wiki(),
            PlayCommand::Grid => self.print_grid(),
            PlayCommand::SetGiven { cells, digit } => self.set_given(cells, digit),
            PlayCommand::Solve { cells, digit } => self.solve_cells(cells, digit),
            PlayCommand::Erase { cells, digits } => self.erase_candidates(cells, digits),
            PlayCommand::Find(filter) => self.find_deductions(filter),
            PlayCommand::Highlight { index } => self.highlight_deduction(index),
            PlayCommand::Apply(index) => self.apply_deductions(index),
            PlayCommand::Verify => self.verify(),
            PlayCommand::Bingo => self.bingo(),
            PlayCommand::Reset => self.reset_candidates(),
            PlayCommand::Undo => self.undo(),
            PlayCommand::Redo => self.redo(),
            PlayCommand::Help => {
                PlayOutput::default().with_overlay(Overlay::Help(play_help_text()))
            }
            PlayCommand::Quit => PlayOutput {
                quit: true,
                ..PlayOutput::default()
            },
        }
    }

    pub fn highlight_all_deductions(&mut self) -> PlayOutput {
        let Some(found) = self.ensure_deductions(self.deductions_strategy) else {
            return PlayOutput::default().message("No deductions found".to_string());
        };

        let mut combined = Action::new(Strategy::Place);
        for action in found.actions() {
            for (cell, digit) in action.collect_sets() {
                combined.set(cell, digit);
            }
            for (cell, digits) in action.collect_erases() {
                combined.erase_digits(cell, digits);
            }
            for (cell, digit, verdict) in action.collect_clues() {
                combined.clue_cell_for_digit(verdict, cell, digit);
            }
        }

        self.highlight = Some(combined);
        let mut output = PlayOutput::default();
        output
            .messages
            .push("Highlighting all deductions".to_string());
        output.show_board = true;
        output
    }

    pub fn create_puzzle_with_progress<F>(&mut self, mut progress: F) -> PlayOutput
    where
        F: FnMut(ProgressStage, usize),
    {
        let mut output = PlayOutput::default();
        let mut generator = Generator::new(false, true);
        match generator.generate_with_progress(&self.changer, |value| {
            progress(ProgressStage::Generate, value)
        }) {
            Some(board) => {
                let mut finder = Finder::new(22, 10, true);
                let (start, _) = finder.backtracking_find_with_progress(board, |value| {
                    progress(ProgressStage::Find, value)
                });
                output.messages.push(format!("==> Clues: {}", start));
                self.push_board(start);
                output.show_board = true;
            }
            None => {
                output
                    .messages
                    .push("==> Failed to create a new puzzle".to_string());
            }
        }
        self.cancelable.clear();
        output
    }

    fn toggle_options(&mut self, flags: Option<String>) -> PlayOutput {
        let mut output = PlayOutput::default();
        if let Some(flags) = flags {
            for c in flags.to_uppercase().chars() {
                match c {
                    'N' => {
                        self.changer.options.solve_naked_singles =
                            !self.changer.options.solve_naked_singles;
                        output.messages.push(format!(
                            "Solving naked singles: {}",
                            on_off(self.changer.options.solve_naked_singles)
                        ));
                    }
                    'H' => {
                        self.changer.options.solve_hidden_singles =
                            !self.changer.options.solve_hidden_singles;
                        output.messages.push(format!(
                            "Solving hidden singles: {}",
                            on_off(self.changer.options.solve_hidden_singles)
                        ));
                    }
                    'I' => {
                        self.changer.options.solve_intersection_removals =
                            !self.changer.options.solve_intersection_removals;
                        output.messages.push(format!(
                            "Solving intersection removals: {}",
                            on_off(self.changer.options.solve_intersection_removals)
                        ));
                    }
                    _ => {
                        output.messages.push(format!("Unknown option: {}", c));
                    }
                }
            }
        } else {
            output.messages.push(format!(
                "Solving naked singles: {}",
                on_off(self.changer.options.solve_naked_singles)
            ));
            output.messages.push(format!(
                "Solving hidden singles: {}",
                on_off(self.changer.options.solve_hidden_singles)
            ));
            output.messages.push(format!(
                "Solving intersection removals: {}",
                on_off(self.changer.options.solve_intersection_removals)
            ));
        }

        output
    }

    fn new_puzzle(&mut self, input: NewPuzzleInput, puzzle: Option<String>) -> PlayOutput {
        match input {
            NewPuzzleInput::Cancel => PlayOutput::default(),
            NewPuzzleInput::Empty => {
                self.push_board(Board::new());
                let mut output = PlayOutput::default();
                output
                    .messages
                    .push("==> Starting an empty puzzle".to_string());
                output.show_board = true;
                output
            }
            NewPuzzleInput::Puzzle => {
                let Some(puzzle) = puzzle else {
                    return PlayOutput::default();
                };
                let input = puzzle.trim();
                if input.is_empty() {
                    return PlayOutput::default();
                }
                if input.eq_ignore_ascii_case("E") {
                    self.push_board(Board::new());
                    let mut output = PlayOutput::default();
                    output
                        .messages
                        .push("==> Starting an empty puzzle".to_string());
                    output.show_board = true;
                    return output;
                }

                let normalized = input.replace(' ', "").replace(MISSING, ".");
                let parser: Option<Box<dyn Parser>> = if normalized.len() >= 160 {
                    Some(Box::new(Parse::wiki()))
                } else if normalized.len() <= 81 {
                    Some(Box::new(Parse::packed_with_player(self.changer)))
                } else {
                    None
                };

                let Some(parser) = parser else {
                    return PlayOutput::default().message(format!(
                        "==> Expected 81 or 162 digits, got {}\n{}",
                        normalized.len(),
                        normalized
                    ));
                };

                let (board, effects, failure) = parser.parse(&normalized);
                self.push_board(board);
                let mut output = PlayOutput::default().with_show_board();
                if let Some((cell, digit)) = failure {
                    output.messages.push(format!(
                        "==> Setting {} to {} will cause errors",
                        cell, digit
                    ));
                    output.messages.extend(errors_to_lines(&effects));
                }
                output
            }
        }
    }

    fn create_puzzle(&mut self) -> PlayOutput {
        let mut output = PlayOutput::default();
        let mut generator = Generator::new(false, true);
        match generator.generate(&self.changer) {
            Some(board) => {
                let mut finder = Finder::new(22, 10, true);
                let (start, _) = finder.backtracking_find(board);
                output.messages.push(format!("==> Clues: {}", start));
                self.push_board(start);
                output.show_board = true;
            }
            None => {
                output
                    .messages
                    .push("==> Failed to create a new puzzle".to_string());
            }
        }
        self.cancelable.clear();
        output
    }

    fn print_board(&mut self, target: Option<PrintTarget>) -> PlayOutput {
        let board = self.current();
        match target {
            Some(PrintTarget::Givens) => PlayOutput::default().with_overlay(Overlay::Board {
                title: "Print",
                lines: add_single_value_labels(crate::io::write_givens(board)),
            }),
            Some(PrintTarget::Solved) => PlayOutput::default().with_overlay(Overlay::Board {
                title: "Print",
                lines: add_single_value_labels(crate::io::write_solved_values(board)),
            }),
            Some(PrintTarget::Candidate(digit)) => {
                PlayOutput::default().with_overlay(Overlay::Board {
                    title: "Print",
                    lines: add_single_value_labels(crate::io::write_candidate(board, digit)),
                })
            }
            None => {
                if board.is_fully_solved() {
                    PlayOutput::default().with_overlay(Overlay::Board {
                        title: "Print",
                        lines: add_single_value_labels(crate::io::write_solved_values(board)),
                    })
                } else {
                    PlayOutput::default().with_show_board()
                }
            }
        }
    }

    fn export_board(&mut self, ch: Option<char>) -> PlayOutput {
        let board = self.current();
        let text = if let Some(ch) = ch {
            format_packed(board, ch, true)
        } else {
            format_for_fancy_console(board)
        };
        PlayOutput::default().with_overlay(Overlay::Text {
            title: "Export",
            lines: text.lines().map(|l| l.to_string()).collect(),
        })
    }

    fn print_wiki(&mut self) -> PlayOutput {
        let board = self.current();
        let text = format!("{}{}", SUDOKUWIKI_URL, format_for_wiki(board));
        PlayOutput::default().with_overlay(Overlay::Text {
            title: "Wiki",
            lines: vec![text],
        })
    }

    fn print_grid(&mut self) -> PlayOutput {
        let board = self.current();
        let text = format_grid(board);
        PlayOutput::default().with_overlay(Overlay::Text {
            title: "Grid",
            lines: text.lines().map(|l| l.to_string()).collect(),
        })
    }

    fn set_given(&mut self, cells: CellSet, digit: Digit) -> PlayOutput {
        let mut output = PlayOutput::default();
        let board = self.current();
        let mut clone = *board;
        let mut changed = false;
        for cell in cells {
            match self.changer.set_given(&clone, Strategy::Give, cell, digit) {
                ChangeResult::None => {
                    output
                        .messages
                        .push(format!("==> {} is not a candidate for {}", digit, cell));
                }
                ChangeResult::Valid(after, _) => {
                    clone = *after;
                    changed = true;
                }
                ChangeResult::Invalid(_, _, _, errors) => {
                    output.messages.push("==> Invalid move".to_string());
                    output.messages.extend(errors_to_lines(&errors).into_iter());
                }
            }
        }
        if changed {
            self.push_board(clone);
            output.show_board = true;
            output.board_changed = true;
        }
        output
    }

    fn solve_cells(&mut self, cells: CellSet, digit: Digit) -> PlayOutput {
        let mut output = PlayOutput::default();
        let board = self.current();
        let mut clone = *board;
        let mut changed = false;
        for cell in cells {
            match self.changer.set_digit(&clone, Strategy::Place, cell, digit) {
                ChangeResult::None => {
                    output
                        .messages
                        .push(format!("==> {} is not a candidate for {}", digit, cell));
                }
                ChangeResult::Valid(after, _) => {
                    clone = *after;
                    changed = true;
                }
                ChangeResult::Invalid(_, _, _, errors) => {
                    output.messages.push("==> Invalid move".to_string());
                    output.messages.extend(errors_to_lines(&errors).into_iter());
                }
            }
        }
        if changed {
            self.push_board(clone);
            output.show_board = true;
            output.board_changed = true;
        }
        output
    }

    fn erase_candidates(&mut self, cells: CellSet, digits: DigitSet) -> PlayOutput {
        let mut output = PlayOutput::default();
        let board = self.current();
        let mut clone = *board;
        let mut changed = false;
        for cell in cells {
            for digit in digits {
                match self
                    .changer
                    .remove_candidate(&clone, Strategy::Erase, cell, digit)
                {
                    ChangeResult::None => {
                        output
                            .messages
                            .push(format!("==> {} is not a candidate for {}", digit, cell));
                    }
                    ChangeResult::Valid(after, _) => {
                        clone = *after;
                        changed = true;
                    }
                    ChangeResult::Invalid(_, _, _, errors) => {
                        output.messages.push("==> Invalid move".to_string());
                        output.messages.extend(errors_to_lines(&errors).into_iter());
                    }
                }
            }
        }
        if changed {
            self.push_board(clone);
            output.show_board = true;
            output.board_changed = true;
        }
        output
    }

    fn find_deductions(&mut self, filter: Option<FindFilter>) -> PlayOutput {
        let strategy_filter = match filter {
            Some(FindFilter::Strategy(strategy)) => Some(strategy),
            _ => None,
        };
        let Some(found) = self.ensure_deductions(strategy_filter) else {
            let message = match filter {
                Some(FindFilter::Cell(cell)) => {
                    format!("==> No deductions found affecting {}", cell)
                }
                Some(FindFilter::Digit(digit)) => {
                    format!("==> No deductions found affecting {}", digit)
                }
                Some(FindFilter::Strategy(strategy)) => {
                    format!("==> No deductions found for {}", strategy.label())
                }
                None => "==> No deductions found".to_string(),
            };
            return PlayOutput::default().message(message);
        };

        let mut lines = Vec::new();
        let message = match filter {
            Some(FindFilter::Cell(cell)) => {
                let filtered = found.affecting_cell(cell);
                if filtered.is_empty() {
                    return PlayOutput::default()
                        .message(format!("==> No deductions found affecting {}", cell));
                }
                Some(format!(
                    "==> Found {} affecting {}",
                    pluralize(filtered.action_count(), "deduction"),
                    cell
                ))
            }
            Some(FindFilter::Digit(digit)) => {
                let filtered = found.affecting_digit(digit);
                if filtered.is_empty() {
                    return PlayOutput::default()
                        .message(format!("==> No deductions found affecting {}", digit));
                }
                Some(format!(
                    "==> Found {} affecting {}",
                    pluralize(filtered.action_count(), "deduction"),
                    digit
                ))
            }
            Some(FindFilter::Strategy(strategy)) => Some(format!(
                "==> Found {} from {}",
                pluralize(found.action_count(), "deduction"),
                strategy.label()
            )),
            None => Some(format!(
                "==> Found {}",
                pluralize(found.action_count(), "deduction")
            )),
        };
        let mut found_any = false;

        for (i, action) in found.actions().iter().enumerate() {
            let mut include = false;
            if let Some(FindFilter::Cell(cell)) = filter {
                if action.affects_cell(cell) {
                    include = true;
                }
            } else if let Some(FindFilter::Digit(digit)) = filter {
                if action.affects_digit(digit) {
                    include = true;
                }
            } else if let Some(FindFilter::Strategy(strategy)) = filter {
                if action.strategy() == strategy {
                    include = true;
                }
            } else {
                include = true;
            }
            if include {
                found_any = true;
                lines.push(format!("{:>4} - {}", i + 1, action));
            }
        }

        if !found_any {
            let msg = match filter {
                Some(FindFilter::Cell(cell)) => {
                    format!("==> No deductions found affecting {}", cell)
                }
                Some(FindFilter::Digit(digit)) => {
                    format!("==> No deductions found affecting {}", digit)
                }
                Some(FindFilter::Strategy(strategy)) => {
                    format!("==> No deductions found for {}", strategy.label())
                }
                None => "==> No deductions found".to_string(),
            };
            return PlayOutput::default().message(msg);
        }

        let mut output = PlayOutput::default();
        if let Some(msg) = message {
            output.messages.push(msg);
        }
        output.overlay = Some(Overlay::Text {
            title: "Find Deductions",
            lines,
        });
        output
    }

    fn highlight_deduction(&mut self, index: usize) -> PlayOutput {
        let mut output = PlayOutput::default();
        let Some(found) = self.ensure_deductions(self.deductions_strategy) else {
            return PlayOutput::default().message("==> Find deductions first with F".to_string());
        };

        if index < 1 || index > found.action_count() {
            return PlayOutput::default().message(format!(
                "==> Enter a deduction number 1 - {}",
                found.action_count()
            ));
        }

        let action = found.actions()[index - 1].clone();
        output.messages.push(format!(
            "==> Highlighting deduction {} - {}",
            index,
            action.strategy().label()
        ));
        self.highlight = Some(action);
        output.show_board = true;
        output
    }

    fn apply_deductions(&mut self, index: Option<usize>) -> PlayOutput {
        let board = *self.current();
        let Some(found) = self.ensure_deductions(self.deductions_strategy) else {
            return PlayOutput::default().message("==> Find deductions first with F".to_string());
        };

        if let Some(index) = index {
            if index < 1 || index > found.action_count() {
                return PlayOutput::default().message(format!(
                    "==> Enter a deduction number 1 - {}",
                    found.action_count()
                ));
            }
            let deduction = found.actions()[index - 1].clone();
            match self.changer.apply(&board, &deduction) {
                ChangeResult::None => {
                    return PlayOutput::default()
                        .message(format!("==> Did not apply {}", deduction));
                }
                ChangeResult::Valid(after, _) => {
                    self.push_board(*after);
                    let mut output = PlayOutput::default();
                    output.messages.push(format!("==> Applied {}", deduction));
                    output.show_board = true;
                    output.board_changed = true;
                    return output;
                }
                ChangeResult::Invalid(_, _, _, errors) => {
                    let mut output = PlayOutput::default();
                    output
                        .messages
                        .push(format!("==> Applying {} will cause errors", deduction));
                    output.messages.extend(errors_to_lines(&errors));
                    return output;
                }
            }
        }

        let mut any_applied = false;
        let mut clone = board;
        let mut output = PlayOutput::default();
        let _ = TECHNIQUES.iter().try_for_each(|solver| {
            if let Some(actions) = solver.solve(&board, false) {
                let mut applied = 0;
                for action in actions.actions() {
                    match self.changer.apply(&clone, action) {
                        ChangeResult::None => (),
                        ChangeResult::Valid(after, _) => {
                            applied += 1;
                            clone = *after;
                        }
                        ChangeResult::Invalid(_, _, _, errors) => {
                            output.messages.push(format!(
                                "==> Applying {} will cause errors\n    {}",
                                solver.label(),
                                action
                            ));
                            output.messages.extend(errors_to_lines(&errors));
                            return Err(());
                        }
                    }
                }
                if applied > 0 {
                    any_applied = true;
                    output.messages.push(format!(
                        "==> Applied {}",
                        pluralize(applied, solver.label())
                    ));
                }
            }
            Ok(())
        });

        if any_applied {
            self.push_board(clone);
            output.show_board = true;
            output.board_changed = true;
        } else if output.messages.is_empty() {
            output
                .messages
                .push("==> No deductions applied".to_string());
        }
        output
    }

    fn verify(&mut self) -> PlayOutput {
        let board = self.current();
        let runtime = Instant::now();
        let mut output = PlayOutput::default();
        match find_brute_force(board, false, 0, MAXIMUM_SOLUTIONS) {
            BruteForceResult::AlreadySolved => {
                output
                    .messages
                    .push("==> The puzzle is already solved".to_string());
            }
            BruteForceResult::TooFewDigits => {
                output
                    .messages
                    .push("==> The puzzle needs at least 17 solved cells to verify".to_string());
            }
            BruteForceResult::UnsolvableCells(cells) => {
                output.messages.push(format!(
                    "==> The puzzle cannot be solved with these {} empty cells\n\n    {}",
                    cells.len(),
                    cells
                ));
            }
            BruteForceResult::Canceled => {
                output.messages.push(format!(
                    "==> The verification was canceled - took {} µs",
                    format_runtime(runtime.elapsed())
                ));
                self.cancelable.clear();
            }
            BruteForceResult::Unsolvable => {
                output.messages.push(format!(
                    "==> The puzzle cannot be solved - took {} µs",
                    format_runtime(runtime.elapsed())
                ));
            }
            BruteForceResult::Solved(_) => {
                output.messages.push(format!(
                    "==> The puzzle is solvable - took {} µs",
                    format_runtime(runtime.elapsed())
                ));
            }
            BruteForceResult::MultipleSolutions(solutions) => {
                output.messages.push(format!(
                    "==> The puzzle has {}{} solutions - took {} µs",
                    if solutions.len() > MAXIMUM_SOLUTIONS {
                        "at least "
                    } else {
                        ""
                    },
                    solutions.len(),
                    format_runtime(runtime.elapsed())
                ));
            }
        }
        output
    }

    fn bingo(&mut self) -> PlayOutput {
        let board = self.current();
        let runtime = Instant::now();
        let mut output = PlayOutput::default();
        match find_brute_force(board, false, 0, MAXIMUM_SOLUTIONS) {
            BruteForceResult::AlreadySolved => {
                output
                    .messages
                    .push("==> The puzzle is already solved".to_string());
            }
            BruteForceResult::TooFewDigits => {
                output
                    .messages
                    .push("==> The puzzle needs at least 17 solved cells to verify".to_string());
            }
            BruteForceResult::UnsolvableCells(cells) => {
                output.messages.push(format!(
                    "==> The puzzle cannot be solved with these {} empty cells\n\n    {}",
                    cells.len(),
                    cells
                ));
            }
            BruteForceResult::Canceled => {
                output.messages.push(format!(
                    "==> The solution was canceled - took {} µs",
                    format_runtime(runtime.elapsed())
                ));
                self.cancelable.clear();
            }
            BruteForceResult::Unsolvable => {
                output.messages.push(format!(
                    "==> The puzzle cannot be solved - took {} µs",
                    format_runtime(runtime.elapsed())
                ));
            }
            BruteForceResult::Solved(solution) => {
                output.messages.push(format!(
                    "==> The puzzle was solved - took {} µs",
                    format_runtime(runtime.elapsed())
                ));
                self.push_board(*solution);
                output.show_board = true;
                output.board_changed = true;
            }
            BruteForceResult::MultipleSolutions(solutions) => {
                output.messages.push(format!(
                    "==> The puzzle has {}{} solutions - took {} µs",
                    if solutions.len() > MAXIMUM_SOLUTIONS {
                        "at least "
                    } else {
                        ""
                    },
                    solutions.len(),
                    format_runtime(runtime.elapsed())
                ));
            }
        }
        output
    }

    fn reset_candidates(&mut self) -> PlayOutput {
        let board = self.current();
        let mut reset = Board::new();
        let mut effects = Effects::new();
        for (cell, digit) in board.solved_iter() {
            reset.set_given(cell, digit, &mut effects);
        }
        if effects.has_errors() {
            let mut output = PlayOutput::default();
            output.messages.push("Invalid board".to_string());
            output.messages.extend(errors_to_lines(&effects));
            return output;
        }

        let mut output = PlayOutput::default();
        if reset != *board {
            output
                .messages
                .push("Reset candidates based on solved cells".to_string());
        }
        self.push_board(reset);
        output.show_board = true;
        output.board_changed = true;
        output
    }

    fn undo(&mut self) -> PlayOutput {
        let mut output = PlayOutput::default();
        if self.index > 0 {
            self.index -= 1;
            self.deductions = None;
            self.deductions_strategy = None;
            self.highlight = None;
            output.messages.push("Undoing last move".to_string());
            output.show_board = true;
        }
        output
    }

    fn ensure_deductions(&mut self, strategy: Option<Strategy>) -> Option<&Effects> {
        if self.deductions.is_none() || self.deductions_strategy != strategy {
            let mut found = Effects::new();
            let board = self.current();
            match strategy {
                Some(target) => {
                    let mut aggregated: Vec<Action> = Vec::new();
                    for solver in TECHNIQUES.iter() {
                        if let Some(actions) = solver.solve(board, false) {
                            take_actions_with_rules(&mut aggregated, actions);
                        }
                        if solver.strategy() == target {
                            break;
                        }
                    }
                    aggregated
                        .iter()
                        .filter(|action| action.has_strategy(target))
                        .for_each(|action| {
                            found.add_action(action.clone());
                        });
                }
                None => {
                    let mut aggregated: Vec<Action> = Vec::new();
                    for solver in TECHNIQUES.iter() {
                        if let Some(actions) = solver.solve(board, false) {
                            take_actions_with_rules(&mut aggregated, actions);
                        }
                    }
                    aggregated.iter().for_each(|action| {
                        found.add_action(action.clone());
                    });
                }
            }
            self.deductions = Some(found);
            self.deductions_strategy = strategy;
        }

        match &self.deductions {
            Some(found) if found.action_count() > 0 => Some(found),
            _ => None,
        }
    }

    fn redo(&mut self) -> PlayOutput {
        let mut output = PlayOutput::default();
        if self.index + 1 < self.boards.len() {
            self.index += 1;
            self.deductions = None;
            self.deductions_strategy = None;
            self.highlight = None;
            output.messages.push("Redoing last move".to_string());
            output.show_board = true;
        }
        output
    }

    fn push_board(&mut self, board: Board) {
        if self.index + 1 < self.boards.len() {
            while self.boards.len() > self.index + 1 {
                self.boards.pop_back();
            }
        }
        self.boards.push_back(board);
        self.index = self.boards.len().saturating_sub(1);
        self.deductions = None;
        self.deductions_strategy = None;
        self.highlight = None;
    }
}

// Used: ABC.EFGH....MNOPQRS..VWXYZ
//
// Want:
// - D for deductions?
// - L for lock candidate(s)
pub fn play_help_text() -> String {
    concat!(
        "==> Help\n",
        "\n",
        "  O [option]                    - view or toggle an option\n",
        "  N                             - start or input a new puzzle\n",
        "  C                             - create a new random puzzle\n",
        "\n",
        "  P [G | S | digit]             - print the full puzzle (or givens, solutions, or single candidate)\n",
        "  X [char]                      - export the puzzle with optional character for unsolved cells\n",
        "  W                             - print URL to play on SudokuWiki.org\n",
        "  M                             - print the puzzle as a grid suitable for email\n",
        "\n",
        "  G <cells> <digit>             - set the given (clue) for the cell(s)\n",
        "  S <cells> <digit>             - solve the cell(s)\n",
        "  E <cells> <digits>            - erase the candidate(s) from the cell(s)\n",
        "\n",
        "  F [cell | digit | strategy]   - find deductions\n",
        "  H <num>                       - highlight a single deduction\n",
        "  A [num]                       - apply a single or all deductions\n",
        "  V                             - verify that puzzle is solvable\n",
        "  B                             - use Bowman's Bingo to solve the puzzle if possible\n",
        "  R                             - reset candidates based on solved cells\n",
        "  Z                             - undo last change\n",
        "  Y                             - redo last change\n",
        "\n",
        "  ?                             - this help message\n",
        "  Q                             - quit\n",
        "\n",
        "          <option> - H, N or I\n",
        "          <cell>   - A1 to J9\n",
        "          <digit>  - 1 to 9\n",
        "          <num>    - any positive number\n",
        "          <char>   - any single character\n",
        "          [...]    - optional\n",
        "\n",
        "  Commands and cells are case-insensitive - \"s a2 4\" and \"E D8 6\" are fine\n",
    )
    .to_string()
}

fn errors_to_lines(effects: &Effects) -> Vec<String> {
    effects
        .errors()
        .iter()
        .map(|error| format!("- {}", error))
        .collect()
}

fn pluralize(count: usize, label: &str) -> String {
    if count == 1 {
        format!("{} {}", count, label)
    } else if ES_SUFFIXES.iter().any(|suffix| label.ends_with(suffix)) {
        format!("{} {}es", count, label)
    } else {
        format!("{} {}s", count, label)
    }
}

const ES_SUFFIXES: [&str; 1] = ["sh"];

fn on_off(value: bool) -> &'static str {
    if value {
        "ON"
    } else {
        "OFF"
    }
}
