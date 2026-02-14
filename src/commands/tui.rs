//! Ratatui-based interactive player.

use std::io::{self, stdout, Write};
use std::time::Duration;
use std::time::Instant;

use clap::Args;
use colored::Colorize;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Terminal;

use crate::io::{
    add_all_candidates_labels, write_candidate, write_candidate_with_highlight, write_candidates,
    write_candidates_with_highlight,
};
use crate::io::{format_for_wiki, SUDOKUWIKI_URL};
use crate::layout::Digit;

use super::play_core::{
    parse_command_line, play_help_text, NewPuzzleInput, Overlay, PlayCommand, PlayOptionsArgs,
    PlayOutput, PlayState, ProgressStage,
};

#[derive(Debug, Args)]
#[clap(disable_help_flag = true)]
pub struct TuiArgs {
    /// Print help information
    #[clap(long, action = clap::ArgAction::HelpLong)]
    help: Option<bool>,

    #[clap(flatten)]
    options: PlayOptionsArgs,

    /// Clues for a starting puzzle
    puzzle: Option<String>,
}

pub fn start_tui(args: TuiArgs) {
    let mut terminal = match init_terminal() {
        Ok(terminal) => terminal,
        Err(err) => {
            eprintln!("Failed to initialize TUI: {}", err);
            return;
        }
    };

    let (mut state, init_output) = PlayState::new(args.options.options(), args.puzzle.clone());
    let mut tui = TuiState::new();
    tui.apply_output(init_output);
    if args.puzzle.is_none() {
        let output = run_create_puzzle_with_progress(&mut terminal, &mut tui, &mut state);
        tui.apply_output(output);
    }

    let mut should_quit = false;
    while !should_quit {
        if let Some(deadline) = tui.overlay_deadline {
            if Instant::now() >= deadline {
                tui.overlay = OverlayState::None;
                tui.overlay_deadline = None;
                tui.clipboard = None;
            }
        }
        if let Err(err) = terminal.draw(|frame| tui.render(frame, &state)) {
            eprintln!("Failed to draw TUI: {}", err);
            break;
        }

        if event::poll(Duration::from_millis(50)).unwrap_or(false) {
            match event::read().unwrap() {
                Event::Resize(_, _) => {
                    let _ = terminal.autoresize();
                }
                Event::Key(key) => {
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && matches!(key.code, KeyCode::Char('c'))
                    {
                        crate::io::Cancelable::new().cancel();
                    }
                    match tui.handle_key(key, &mut state) {
                        TuiAction::None => {}
                        TuiAction::Applied(output) => {
                            if output.quit {
                                should_quit = true;
                            } else {
                                tui.apply_output(output);
                            }
                        }
                        TuiAction::CreatePuzzle => {
                            let output = run_create_puzzle_with_progress(
                                &mut terminal,
                                &mut tui,
                                &mut state,
                            );
                            tui.apply_output(output);
                        }
                        TuiAction::Wiki => {
                            let output = copy_puzzle_to_clipboard(&mut tui, &state);
                            tui.apply_output(output);
                            tui.overlay_deadline = Some(Instant::now() + Duration::from_secs(2));
                        }
                        TuiAction::Grid => {
                            let output = copy_grid_to_clipboard(&mut tui, &mut state);
                            tui.apply_output(output);
                        }
                        TuiAction::Quit => should_quit = true,
                    }
                }
                _ => {}
            }
        }
    }

    let _ = restore_terminal(terminal);
}

enum TuiAction {
    None,
    Applied(PlayOutput),
    CreatePuzzle,
    Wiki,
    Grid,
    Quit,
}

#[derive(Clone, Copy)]
enum ArgKind {
    Cells,
    Digit,
    Digits,
    Number,
    OptionFlags,
    Char,
    Puzzle,
    CellOrDigit,
    Print,
}

struct CommandSpec {
    key: char,
    name: &'static str,
    args: Vec<ArgKind>,
    labels: Vec<&'static str>,
    allow_empty: bool,
}

struct InputState {
    spec: CommandSpec,
    buffer: String,
    error: Option<String>,
}

enum OverlayState {
    None,
    Input(InputState),
    Help(String),
    Board { title: String, lines: Vec<String> },
    Text { title: String, lines: Vec<String> },
    Progress { title: String, lines: Vec<String> },
}

struct TuiState {
    overlay: OverlayState,
    overlay_deadline: Option<Instant>,
    status_message: Option<String>,
    clipboard: Option<arboard::Clipboard>,
}

impl TuiState {
    fn new() -> Self {
        Self {
            overlay: OverlayState::None,
            overlay_deadline: None,
            status_message: None,
            clipboard: None,
        }
    }

    fn apply_output(&mut self, output: PlayOutput) {
        if output.messages.is_empty() {
            self.status_message = None;
        } else {
            let first = output
                .messages
                .iter()
                .flat_map(|message| message.lines())
                .next()
                .map(|line| clean_line(line.to_string()));
            self.status_message = first;
        }
        if let Some(overlay) = output.overlay {
            self.overlay = match overlay {
                Overlay::Help(text) => OverlayState::Help(text),
                Overlay::Board { title, lines } => OverlayState::Board {
                    title: title.to_string(),
                    lines,
                },
                Overlay::Text { title, lines } => OverlayState::Text {
                    title: title.to_string(),
                    lines: lines.into_iter().map(clean_line).collect(),
                },
            };
            self.overlay_deadline = None;
        } else {
            self.overlay = OverlayState::None;
            self.overlay_deadline = None;
        }
    }

    fn handle_key(&mut self, key: KeyEvent, state: &mut PlayState) -> TuiAction {
        match &mut self.overlay {
            OverlayState::Input(input) => match key.code {
                KeyCode::Esc => {
                    self.overlay = OverlayState::None;
                    TuiAction::None
                }
                KeyCode::Enter => {
                    let buffer = input.buffer.trim().to_string();
                    if buffer.is_empty() && !input.spec.allow_empty {
                        if input.spec.key == 'H' {
                            self.overlay = OverlayState::None;
                            return TuiAction::Applied(state.highlight_all_deductions());
                        }
                        self.overlay = OverlayState::None;
                        return TuiAction::None;
                    }
                    if let Some(error) = input_error(&input.spec, &buffer) {
                        input.error = Some(error);
                        beep();
                        return TuiAction::None;
                    }
                    let command = match input.spec.key {
                        'N' => {
                            if buffer.is_empty() {
                                PlayCommand::NewPuzzle {
                                    input: NewPuzzleInput::Empty,
                                    puzzle: None,
                                }
                            } else {
                                PlayCommand::NewPuzzle {
                                    input: NewPuzzleInput::Puzzle,
                                    puzzle: Some(buffer),
                                }
                            }
                        }
                        _ => {
                            let line = if buffer.is_empty() {
                                input.spec.key.to_string()
                            } else {
                                format!("{} {}", input.spec.key, buffer)
                            };
                            match parse_command_line(&line) {
                                Ok(command) => command,
                                Err(err) => {
                                    input.error = Some(clean_line(err.to_string()));
                                    return TuiAction::None;
                                }
                            }
                        }
                    };

                    self.overlay = OverlayState::None;
                    TuiAction::Applied(state.apply(command))
                }
                KeyCode::Backspace => {
                    input.buffer.pop();
                    input.error = input_error(&input.spec, &input.buffer);
                    TuiAction::None
                }
                KeyCode::Char(ch) => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        return TuiAction::None;
                    }
                    if let Some(ch) = normalize_char(ch) {
                        if input.spec.key == 'O' && input.buffer.is_empty() {
                            let token = ch.to_string();
                            if token_valid(ArgKind::OptionFlags, &token) {
                                let line = format!("O {}", token);
                                if let Ok(command) = parse_command_line(&line) {
                                    self.overlay = OverlayState::None;
                                    return TuiAction::Applied(state.apply(command));
                                }
                            }
                        }
                        if input.spec.key == 'P' && input.buffer.is_empty() {
                            let token = ch.to_string();
                            if token_valid(ArgKind::Print, &token) {
                                let line = format!("P {}", token);
                                if let Ok(command) = parse_command_line(&line) {
                                    self.overlay = OverlayState::None;
                                    return TuiAction::Applied(state.apply(command));
                                }
                            }
                        }
                        if input.spec.key == 'X' && input.buffer.is_empty() {
                            let token = ch.to_string();
                            if token_valid(ArgKind::Char, &token) {
                                let line = format!("X {}", token);
                                if let Ok(command) = parse_command_line(&line) {
                                    self.overlay = OverlayState::None;
                                    return TuiAction::Applied(state.apply(command));
                                }
                            }
                        }
                        let next = format!("{}{}", input.buffer, ch);
                        if validate_buffer(&input.spec, &next) {
                            input.buffer.push(ch);
                            input.error = input_error(&input.spec, &input.buffer);
                        } else {
                            input.error = Some("Invalid input".to_string());
                            beep();
                        }
                    }
                    TuiAction::None
                }
                _ => TuiAction::None,
            },
            OverlayState::Help(_)
            | OverlayState::Board { .. }
            | OverlayState::Text { .. }
            | OverlayState::Progress { .. } => {
                self.overlay = OverlayState::None;
                self.overlay_deadline = None;
                self.clipboard = None;
                if matches!(key.code, KeyCode::Esc) {
                    TuiAction::None
                } else {
                    self.handle_idle_key(key, state)
                }
            }
            OverlayState::None => self.handle_idle_key(key, state),
        }
    }

    fn handle_idle_key(&mut self, key: KeyEvent, state: &mut PlayState) -> TuiAction {
        match key.code {
            KeyCode::Esc => TuiAction::None,
            KeyCode::Char('?') => {
                self.overlay = OverlayState::Help(play_help_text());
                TuiAction::None
            }
            KeyCode::Char(ch) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    return TuiAction::None;
                }
                let command = ch.to_ascii_uppercase();
                if command == 'C' {
                    return TuiAction::CreatePuzzle;
                }
                if command == 'W' {
                    return TuiAction::Wiki;
                }
                if command == 'M' {
                    return TuiAction::Grid;
                }
                if let Some(spec) = command_spec(command) {
                    self.overlay = OverlayState::Input(InputState {
                        spec,
                        buffer: String::new(),
                        error: None,
                    });
                    return TuiAction::None;
                }

                let line = command.to_string();
                match parse_command_line(&line) {
                    Ok(command) => TuiAction::Applied(state.apply(command)),
                    Err(_) => TuiAction::None,
                }
            }
            _ => TuiAction::None,
        }
    }

    fn render(&mut self, frame: &mut ratatui::Frame, state: &PlayState) {
        let size = frame.size();

        let main_area = Rect {
            x: size.x,
            y: size.y,
            width: size.width,
            height: size.height.saturating_sub(3),
        };
        let (left_area, right_area) = split_main_areas(state, main_area);
        self.render_main(frame, state, left_area, right_area);
        self.render_status(
            frame,
            Rect {
                x: size.x,
                y: size.y + size.height.saturating_sub(3),
                width: size.width,
                height: 3,
            },
        );

        match &self.overlay {
            OverlayState::Input(input) => self.render_input_overlay(frame, input, size),
            OverlayState::Help(text) => self.render_help_overlay(frame, text, size),
            OverlayState::Board { title, lines } => {
                self.render_lines_overlay(frame, title, lines, size)
            }
            OverlayState::Text { title, lines } => {
                self.render_lines_overlay(frame, title, lines, size)
            }
            OverlayState::Progress { title, lines } => {
                self.render_lines_overlay(frame, title, lines, size)
            }
            OverlayState::None => {}
        }
    }

    fn render_status(&self, frame: &mut ratatui::Frame, area: Rect) {
        let line = self.status_message.clone().unwrap_or_default();
        let inner = inner_rect(area);
        let pad = inner
            .width
            .saturating_sub(line.chars().count() as u16)
            .saturating_div(2) as usize;
        let text = Text::from(Line::from(vec![
            Span::raw(" ".repeat(pad)),
            Span::raw(line),
        ]));
        let widget = Paragraph::new(text)
            .block(Block::default().title(" Status ").borders(Borders::ALL))
            .wrap(Wrap { trim: false });
        frame.render_widget(widget, area);
    }

    fn render_main(
        &self,
        frame: &mut ratatui::Frame,
        state: &PlayState,
        left_area: Rect,
        right_area: Rect,
    ) {
        let board = state.current();
        let grid = if let Some(action) = state.highlight() {
            add_all_candidates_labels(write_candidates_with_highlight(
                board,
                action.collect_verdicts(),
            ))
        } else {
            add_all_candidates_labels(strip_ansi_lines(write_candidates(board)))
        };
        let status = format!(
            "[ {} solved / {} unsolved ]",
            board.solved_count(),
            board.unsolved_count()
        );
        let grid_width = grid
            .iter()
            .map(|line| visible_len(&trim_ansi_trailing_spaces(line)))
            .max()
            .unwrap_or(0);
        let status_pad = grid_width.saturating_sub(status.chars().count()) / 2;
        let status_line = if status_pad > 0 {
            format!("{:width$}{}", "", status, width = status_pad)
        } else {
            status
        };
        let mut grid_block = grid.clone();
        grid_block.push(String::new());
        grid_block.push(status_line);
        let left_inner = inner_rect(left_area);
        let centered_grid = if state.highlight().is_some() {
            center_lines_visible(&grid_block, left_inner)
        } else {
            center_lines(&grid_block, left_inner)
        };

        let left_text = if state.highlight().is_some() {
            ansi_lines_to_text(&centered_grid)
        } else {
            Text::from(centered_grid.join("\n"))
        };
        let left = Paragraph::new(left_text)
            .block(Block::default().title(" Board ").borders(Borders::ALL))
            .wrap(Wrap { trim: false });
        frame.render_widget(left, left_area);

        let right_inner = inner_rect(right_area);
        let right_base_width = candidate_grid_width(board) * 3;
        let right_spacing = choose_column_spacing(right_inner.width, right_base_width);
        let panel_width = right_inner.width as usize;
        let panel_height = right_inner.height as usize;
        let candidate_lines = build_candidate_panel(
            board,
            state.highlight(),
            panel_width,
            panel_height,
            right_spacing,
        );
        let centered_candidates = center_lines_visible(&candidate_lines, right_inner);
        let right_text = if let Some(action) = state.highlight() {
            let highlighted = build_candidate_panel_with_highlight(
                board,
                action,
                panel_width,
                panel_height,
                right_spacing,
            );
            let centered = center_lines_visible(&highlighted, right_inner);
            ansi_lines_to_text(&centered)
        } else {
            ansi_lines_to_text(&centered_candidates)
        };
        let right = Paragraph::new(right_text)
            .block(Block::default().title(" Candidates ").borders(Borders::ALL));
        frame.render_widget(right, right_area);
    }

    fn render_input_overlay(&self, frame: &mut ratatui::Frame, input: &InputState, size: Rect) {
        let typed = input.buffer.clone();
        let prompt_hint = remaining_prompt(&input.spec, &input.buffer);
        let display_len = typed.chars().count() + prompt_hint.chars().count();
        let (lines, display_width, cursor_pos) =
            input_overlay_lines(input, &typed, &prompt_hint, display_len);
        let area = input_overlay_rect(display_width, lines.len(), size);
        frame.render_widget(Clear, area);

        let widget = Paragraph::new(Text::from(lines.clone()))
            .block(
                Block::default()
                    .title(format!(" {} ", input.spec.name))
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(widget, area);

        frame.set_cursor(
            area.x + 1 + cursor_pos.0 as u16,
            area.y + 1 + cursor_pos.1 as u16,
        );
    }

    fn render_help_overlay(&self, frame: &mut ratatui::Frame, text: &str, size: Rect) {
        let mut raw_lines = text.lines().collect::<Vec<_>>();
        if raw_lines
            .first()
            .map(|line| line.starts_with("==> Help"))
            .unwrap_or(false)
        {
            raw_lines.remove(0);
        }
        let lines = raw_lines
            .into_iter()
            .map(|l| l.to_string())
            .collect::<Vec<_>>();
        let padded = pad_lines(&lines, 2, 1);
        let (width, height) = overlay_dimensions(&padded, size);
        let area = overlay_rect_with_inset(size, width, height);
        frame.render_widget(Clear, area);
        let text = Text::from(padded.join("\n"));
        let widget = Paragraph::new(text)
            .block(Block::default().title(" Help ").borders(Borders::ALL))
            .wrap(Wrap { trim: false });
        frame.render_widget(widget, area);
    }

    fn render_lines_overlay(
        &self,
        frame: &mut ratatui::Frame,
        title: &str,
        lines: &[String],
        size: Rect,
    ) {
        let padded = pad_lines(lines, 2, 1);
        let (width, height) = overlay_dimensions(&padded, size);
        let area = overlay_rect_with_inset(size, width, height);
        frame.render_widget(Clear, area);
        let text = Text::from(padded.join("\n"));
        let widget = Paragraph::new(text)
            .block(
                Block::default()
                    .title(format!(" {} ", title))
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(widget, area);
    }
}

fn build_candidate_panel(
    board: &crate::puzzle::Board,
    _highlight: Option<&crate::puzzle::Action>,
    _available_width: usize,
    available_height: usize,
    spacing: usize,
) -> Vec<String> {
    let mut grids = Vec::new();
    for digit in Digit::iter() {
        let grid = write_candidate(board, digit);
        grids.push(gray_if_complete(board, digit, grid));
    }

    let mut columns = [Vec::new(), Vec::new(), Vec::new()];
    for (i, grid) in grids.iter().enumerate() {
        if !columns[i % 3].is_empty() {
            columns[i % 3].push(String::new());
        }
        columns[i % 3].extend(grid.iter().map(|line| trim_ansi_trailing_spaces(line)));
    }

    let widths = [
        columns[0].iter().map(|l| visible_len(l)).max().unwrap_or(0),
        columns[1].iter().map(|l| visible_len(l)).max().unwrap_or(0),
        columns[2].iter().map(|l| visible_len(l)).max().unwrap_or(0),
    ];
    let max_len = columns.iter().map(|c| c.len()).max().unwrap_or(0);

    let sep = if spacing == 2 { "  " } else { " " };
    let gap = row_spacing(
        grids.first().map(|g| g.len()).unwrap_or(0),
        available_height,
    );
    let mut lines = Vec::new();
    for i in 0..max_len {
        let a = columns[0].get(i).cloned().unwrap_or_default();
        let b = columns[1].get(i).cloned().unwrap_or_default();
        let c = columns[2].get(i).cloned().unwrap_or_default();
        lines.push(format!(
            "{}{}{}{}{}",
            pad_ansi_line(&a, widths[0]),
            sep,
            pad_ansi_line(&b, widths[1]),
            sep,
            pad_ansi_line(&c, widths[2])
        ));
    }

    if gap > 1 {
        insert_row_gaps(lines, gap - 1)
    } else {
        lines
    }
}

fn build_candidate_panel_with_highlight(
    board: &crate::puzzle::Board,
    action: &crate::puzzle::Action,
    _available_width: usize,
    available_height: usize,
    spacing: usize,
) -> Vec<String> {
    let mut grids = Vec::new();
    for digit in Digit::iter() {
        let grid = if digit_complete(board, digit) {
            gray_lines(write_candidate(board, digit))
        } else {
            write_candidate_with_highlight(board, digit, action.collect_verdicts_for_digit(digit))
        };
        grids.push(grid);
    }

    let mut columns = [Vec::new(), Vec::new(), Vec::new()];
    for (i, grid) in grids.iter().enumerate() {
        if !columns[i % 3].is_empty() {
            columns[i % 3].push(String::new());
        }
        columns[i % 3].extend(grid.iter().map(|line| trim_ansi_trailing_spaces(line)));
    }

    let widths = [
        columns[0].iter().map(|l| visible_len(l)).max().unwrap_or(0),
        columns[1].iter().map(|l| visible_len(l)).max().unwrap_or(0),
        columns[2].iter().map(|l| visible_len(l)).max().unwrap_or(0),
    ];
    let max_len = columns.iter().map(|c| c.len()).max().unwrap_or(0);

    let sep = if spacing == 2 { "  " } else { " " };
    let gap = row_spacing(
        grids.first().map(|g| g.len()).unwrap_or(0),
        available_height,
    );
    let mut lines = Vec::new();
    for i in 0..max_len {
        let a = columns[0].get(i).cloned().unwrap_or_default();
        let b = columns[1].get(i).cloned().unwrap_or_default();
        let c = columns[2].get(i).cloned().unwrap_or_default();
        lines.push(format!(
            "{}{}{}{}{}",
            pad_ansi_line(&a, widths[0]),
            sep,
            pad_ansi_line(&b, widths[1]),
            sep,
            pad_ansi_line(&c, widths[2])
        ));
    }

    if gap > 1 {
        insert_row_gaps(lines, gap - 1)
    } else {
        lines
    }
}

fn remaining_prompt(spec: &CommandSpec, buffer: &str) -> String {
    if spec.labels.is_empty() {
        return String::new();
    }
    let tokens = buffer.split_whitespace().collect::<Vec<_>>();
    let start = tokens.len().min(spec.labels.len());
    let remaining = spec.labels[start..].join(" ");
    if remaining.is_empty() {
        return String::new();
    }
    if buffer.is_empty() || buffer.ends_with(' ') {
        remaining.to_string()
    } else {
        format!(" {}", remaining)
    }
}

fn input_error(spec: &CommandSpec, buffer: &str) -> Option<String> {
    let trimmed = buffer.trim();
    if trimmed.is_empty() {
        return None;
    }
    let tokens = trimmed.split_whitespace().collect::<Vec<_>>();
    for (index, token) in tokens.iter().enumerate() {
        let kind = spec.args[index];
        if !token_valid(kind, token) {
            return Some("Invalid input".to_string());
        }
    }
    if tokens.len() < spec.args.len() {
        return None;
    }
    let line = format!("{} {}", spec.key, trimmed);
    match parse_command_line(&line) {
        Ok(_) => None,
        Err(err) => Some(clean_line(err.to_string())),
    }
}

fn input_overlay_lines(
    input: &InputState,
    typed: &str,
    prompt_hint: &str,
    display_len: usize,
) -> (Vec<Line<'static>>, usize, (usize, usize)) {
    let mut raw_lines: Vec<(Line<'static>, usize)> = Vec::new();
    let mut input_line_index = 0usize;

    raw_lines.push((Line::from(""), 0));
    input_line_index += 1;

    if input.spec.key == 'N' {
        let instructions = vec![
            "81 digits",
            "use period or zero to leave a cell blank",
            "spaces are ignored",
            "leave empty for an empty puzzle",
        ];
        for line in instructions {
            let len = line.chars().count();
            raw_lines.push((Line::from(Span::raw(line.to_string())), len));
            input_line_index += 1;
        }
        raw_lines.push((Line::from(""), 0));
        input_line_index += 1;
    } else if input.spec.key == 'O' {
        let line = "N naked | H hidden | I intersection";
        raw_lines.push((
            Line::from(Span::raw(line.to_string())),
            line.chars().count(),
        ));
        raw_lines.push((Line::from(""), 0));
        input_line_index += 2;
    } else if input.spec.key == 'P' {
        let line = "G givens | S solved | 1-9 digit";
        raw_lines.push((
            Line::from(Span::raw(line.to_string())),
            line.chars().count(),
        ));
        raw_lines.push((Line::from(""), 0));
        input_line_index += 2;
    } else {
        raw_lines.push((Line::from(""), 0));
        input_line_index += 1;
    }

    let hint_span = if prompt_hint.is_empty() {
        Span::raw("")
    } else {
        Span::styled(
            prompt_hint.to_string(),
            Style::default().fg(Color::DarkGray),
        )
    };
    let input_line = Line::from(vec![Span::raw(typed.to_string()), hint_span]);
    raw_lines.push((input_line, display_len));
    let input_index = input_line_index;

    raw_lines.push((Line::from(""), 0));

    let error_text = input.error.clone().unwrap_or_default();
    let error_len = error_text.chars().count();
    let error_line = if error_text.is_empty() {
        Line::from("")
    } else {
        Line::from(Span::styled(
            error_text,
            Style::default().fg(Color::LightRed),
        ))
    };
    raw_lines.push((error_line, error_len));

    raw_lines.push((Line::from(""), 0));

    let max_len = raw_lines.iter().map(|(_, len)| *len).max().unwrap_or(0);
    let width = input_overlay_inner_width(max_len.max(display_len));
    let mut lines = Vec::new();
    let mut input_cursor = (0usize, input_index);
    for (index, (line, len)) in raw_lines.into_iter().enumerate() {
        let pad = width.saturating_sub(len) / 2;
        let mut spans = Vec::new();
        if pad > 0 {
            spans.push(Span::raw(" ".repeat(pad)));
        }
        spans.extend(line.into_iter());
        lines.push(Line::from(spans));
        if index == input_index {
            input_cursor.0 = pad + typed.chars().count();
            input_cursor.1 = index;
        }
    }

    (lines, width, input_cursor)
}

fn split_main_areas(state: &PlayState, main: Rect) -> (Rect, Rect) {
    let board = state.current();
    let left_grid_width = left_grid_max_width(board, state.highlight());
    let right_grid_width = candidate_grid_width(board);

    let left_content = left_grid_width;
    let right_content = right_grid_width * 3 + 2;
    let left_min = (left_content + 2) as u16;
    let right_min = (right_content + 2) as u16;
    let total = main.width;

    let (left_width, right_width) = if total >= left_min + right_min {
        let extra = total - left_min - right_min;
        let weight_sum = left_content.max(1) + right_content.max(1);
        let left_extra = (extra as usize * left_content.max(1) / weight_sum) as u16;
        let right_extra = extra - left_extra;
        (left_min + left_extra, right_min + right_extra)
    } else if total == 0 {
        (0, 0)
    } else {
        let left_width =
            (total as usize * left_min as usize / (left_min + right_min) as usize).max(1) as u16;
        let right_width = total.saturating_sub(left_width);
        (left_width, right_width)
    };

    let left = Rect {
        x: main.x,
        y: main.y,
        width: left_width,
        height: main.height,
    };
    let right = Rect {
        x: main.x + left_width,
        y: main.y,
        width: right_width,
        height: main.height,
    };
    (left, right)
}

fn left_grid_max_width(
    board: &crate::puzzle::Board,
    highlight: Option<&crate::puzzle::Action>,
) -> usize {
    let grid = if let Some(action) = highlight {
        add_all_candidates_labels(write_candidates_with_highlight(
            board,
            action.collect_verdicts(),
        ))
    } else {
        add_all_candidates_labels(strip_ansi_lines(write_candidates(board)))
    };
    grid.iter()
        .map(|line| visible_len(line.trim_end_matches(' ')))
        .max()
        .unwrap_or(0)
}

fn candidate_grid_width(board: &crate::puzzle::Board) -> usize {
    let grid = write_candidate(board, Digit::from_ordinal(1));
    grid.iter()
        .map(|line| line.trim_end_matches(' ').chars().count())
        .max()
        .unwrap_or(0)
}

fn candidate_grid_height(board: &crate::puzzle::Board) -> usize {
    let grid = write_candidate(board, Digit::from_ordinal(1));
    grid.len()
}

fn input_overlay_rect(inner_width: usize, line_count: usize, size: Rect) -> Rect {
    let mut width = inner_width as u16 + 2;
    let max_width = size.width.saturating_sub(2).max(30);
    if width > max_width {
        width = max_width;
    }
    let height = (line_count as u16 + 2).min(size.height.saturating_sub(2));
    centered_rect_exact(width, height, size)
}

fn overlay_dimensions(lines: &[String], size: Rect) -> (u16, u16) {
    let max_len = lines.iter().map(|l| l.len()).max().unwrap_or(0) as u16;
    let width = (max_len + 2).min(size.width.saturating_sub(2));
    let height = (lines.len() as u16 + 2).min(size.height.saturating_sub(2));
    (width, height)
}

fn input_overlay_inner_width(content_len: usize) -> usize {
    let desired = content_len.saturating_add(4).max(30);
    10 * desired.div_ceil(10)
}

fn overlay_rect_with_inset(size: Rect, width: u16, height: u16) -> Rect {
    let inset = overlay_inset(size);
    let max_width = size.width.saturating_sub(inset * 2).max(4);
    let max_height = size.height.saturating_sub(inset * 2).max(4);
    let width = width.min(max_width);
    let height = height.min(max_height);
    let rect = Rect {
        x: size.x + inset,
        y: size.y + inset,
        width: max_width,
        height: max_height,
    };
    centered_rect_exact(width, height, rect)
}

fn overlay_inset(size: Rect) -> u16 {
    if size.width >= 120 || size.height >= 40 {
        2
    } else {
        1
    }
}

fn pad_lines(lines: &[String], pad_h: usize, pad_v: usize) -> Vec<String> {
    let mut padded = Vec::new();
    for _ in 0..pad_v {
        padded.push(String::new());
    }
    for line in lines {
        padded.push(format!(
            "{}{}{}",
            " ".repeat(pad_h),
            line,
            " ".repeat(pad_h)
        ));
    }
    for _ in 0..pad_v {
        padded.push(String::new());
    }
    padded
}

fn center_lines(lines: &[String], inner: Rect) -> Vec<String> {
    if inner.width == 0 || inner.height == 0 {
        return lines.to_vec();
    }
    let width = inner.width as usize;
    let height = inner.height as usize;
    let max_len = lines
        .iter()
        .map(|line| line.trim_end_matches(' ').chars().count())
        .max()
        .unwrap_or(0);
    let left_pad = width.saturating_sub(max_len) / 2;
    let total = lines.len();
    let top_pad = height.saturating_sub(total) / 2;
    let mut centered = Vec::with_capacity(height.max(total));
    for _ in 0..top_pad {
        centered.push(String::new());
    }
    for line in lines {
        let trimmed = line.trim_end_matches(' ');
        if max_len >= width {
            centered.push(trimmed.to_string());
        } else {
            let padded = format!("{:width$}{}", "", trimmed, width = left_pad);
            centered.push(format!("{:<width$}", padded, width = width));
        }
    }
    let remaining = height.saturating_sub(centered.len());
    for _ in 0..remaining {
        centered.push(String::new());
    }
    centered
}

fn center_lines_visible(lines: &[String], inner: Rect) -> Vec<String> {
    if inner.width == 0 || inner.height == 0 {
        return lines.to_vec();
    }
    let width = inner.width as usize;
    let height = inner.height as usize;
    let max_len = lines
        .iter()
        .map(|line| visible_len(&trim_ansi_trailing_spaces(line)))
        .max()
        .unwrap_or(0);
    let left_pad = width.saturating_sub(max_len) / 2;
    let total = lines.len();
    let top_pad = height.saturating_sub(total) / 2;
    let mut centered = Vec::with_capacity(height.max(total));
    for _ in 0..top_pad {
        centered.push(String::new());
    }
    for line in lines {
        let trimmed = trim_ansi_trailing_spaces(line);
        if max_len >= width {
            centered.push(trimmed);
        } else {
            let padded = format!("{:width$}{}", "", trimmed, width = left_pad);
            centered.push(pad_ansi_line(&padded, width));
        }
    }
    let remaining = height.saturating_sub(centered.len());
    for _ in 0..remaining {
        centered.push(String::new());
    }
    centered
}

fn inner_rect(area: Rect) -> Rect {
    if area.width < 2 || area.height < 2 {
        return area;
    }
    Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width - 2,
        height: area.height - 2,
    }
}

fn centered_rect_exact(width: u16, height: u16, rect: Rect) -> Rect {
    let width = width.min(rect.width);
    let height = height.min(rect.height);
    let x = rect.x + (rect.width.saturating_sub(width)) / 2;
    let y = rect.y + (rect.height.saturating_sub(height)) / 2;
    Rect {
        x,
        y,
        width,
        height,
    }
}

fn command_spec(key: char) -> Option<CommandSpec> {
    let spec = match key {
        'O' => CommandSpec {
            key,
            name: "Options",
            args: vec![ArgKind::OptionFlags],
            labels: vec!["option"],
            allow_empty: true,
        },
        'N' => CommandSpec {
            key,
            name: "New Puzzle",
            args: vec![ArgKind::Puzzle],
            labels: vec!["givens"],
            allow_empty: true,
        },
        'G' => CommandSpec {
            key,
            name: "Set Given",
            args: vec![ArgKind::Cells, ArgKind::Digit],
            labels: vec!["cells", "digit"],
            allow_empty: false,
        },
        'S' => CommandSpec {
            key,
            name: "Solve Cells",
            args: vec![ArgKind::Cells, ArgKind::Digit],
            labels: vec!["cells", "digit"],
            allow_empty: false,
        },
        'E' => CommandSpec {
            key,
            name: "Erase Candidates",
            args: vec![ArgKind::Cells, ArgKind::Digits],
            labels: vec!["cells", "digits"],
            allow_empty: false,
        },
        'F' => CommandSpec {
            key,
            name: "Find Deductions",
            args: vec![ArgKind::CellOrDigit],
            labels: vec!["cell or digit"],
            allow_empty: true,
        },
        'H' => CommandSpec {
            key,
            name: "Highlight Deduction",
            args: vec![ArgKind::Number],
            labels: vec!["num"],
            allow_empty: false,
        },
        'A' => CommandSpec {
            key,
            name: "Apply Deductions",
            args: vec![ArgKind::Number],
            labels: vec!["num"],
            allow_empty: true,
        },
        'P' => CommandSpec {
            key,
            name: "Print",
            args: vec![ArgKind::Print],
            labels: vec!["G, S, or digit"],
            allow_empty: true,
        },
        'X' => CommandSpec {
            key,
            name: "Export",
            args: vec![ArgKind::Char],
            labels: vec!["char"],
            allow_empty: true,
        },
        _ => return None,
    };
    Some(spec)
}

fn validate_buffer(spec: &CommandSpec, buffer: &str) -> bool {
    if buffer.trim().is_empty() {
        return spec.allow_empty;
    }
    if spec.args.len() == 1 && matches!(spec.args[0], ArgKind::Puzzle) {
        return buffer
            .chars()
            .all(|c| matches!(c, '0'..='9' | '.' | '·' | ' '));
    }
    let tokens = buffer.split_whitespace().collect::<Vec<_>>();
    if tokens.len() > spec.args.len() {
        return false;
    }
    for (index, token) in tokens.iter().enumerate() {
        let kind = spec.args[index];
        if !token_valid(kind, token) {
            return false;
        }
    }
    true
}

fn token_valid(kind: ArgKind, token: &str) -> bool {
    match kind {
        ArgKind::Cells => cells_token_valid(token),
        ArgKind::Digit => token.len() <= 1 && token.chars().all(|c| matches!(c, '1'..='9')),
        ArgKind::Digits => token.chars().all(|c| matches!(c, '1'..='9')),
        ArgKind::Number => token.chars().all(|c| c.is_ascii_digit()),
        ArgKind::OptionFlags => token.chars().all(|c| matches!(c, 'N' | 'H' | 'I')),
        ArgKind::Char => token.len() <= 1 && !token.is_empty(),
        ArgKind::Puzzle => token.chars().all(|c| matches!(c, '0'..='9' | '.' | '·')),
        ArgKind::CellOrDigit => token.len() <= 2 && token.chars().all(is_cell_char),
        ArgKind::Print => {
            token.len() <= 1 && token.chars().all(|c| matches!(c, 'G' | 'S' | '1'..='9'))
        }
    }
}

fn is_cell_char(ch: char) -> bool {
    matches!(ch, '1'..='9' | 'A'..='H' | 'J')
}

fn normalize_char(ch: char) -> Option<char> {
    if ch.is_ascii_control() {
        None
    } else if ch.is_ascii_lowercase() {
        Some(ch.to_ascii_uppercase())
    } else {
        Some(ch)
    }
}

fn cells_token_valid(token: &str) -> bool {
    let mut expect_letter = true;
    for ch in token.chars() {
        if expect_letter {
            if !matches!(ch, 'A'..='H' | 'J') {
                return false;
            }
        } else if !matches!(ch, '1'..='9') {
            return false;
        }
        expect_letter = !expect_letter;
    }
    true
}

fn beep() {
    let _ = std::io::stdout().write_all(b"\x07");
    let _ = std::io::stdout().flush();
}

fn clean_line(line: String) -> String {
    line.strip_prefix("==> ").unwrap_or(&line).to_string()
}

fn copy_puzzle_to_clipboard(tui: &mut TuiState, state: &PlayState) -> PlayOutput {
    let url = format!("{}{}", SUDOKUWIKI_URL, format_for_wiki(state.current()));
    let mut output = PlayOutput::default();
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        if clipboard.set_text(url).is_ok() {
            tui.clipboard = Some(clipboard);
            output.messages.push("Sudokuwiki URL copied".to_string());
            output.overlay = Some(Overlay::Text {
                title: "Clipboard",
                lines: vec!["Copied puzzle to clipboard".to_string()],
            });
            return output;
        }
    }

    output.messages.push("Unable to copy puzzle".to_string());
    output.overlay = Some(Overlay::Text {
        title: "Clipboard",
        lines: vec!["Unable to copy puzzle to clipboard".to_string()],
    });
    output
}

fn copy_grid_to_clipboard(tui: &mut TuiState, state: &mut PlayState) -> PlayOutput {
    let mut output = state.apply(PlayCommand::Grid);
    let grid_text = match &output.overlay {
        Some(Overlay::Text { lines, .. }) => lines.join("\n"),
        _ => String::new(),
    };
    if !grid_text.is_empty() {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            if clipboard.set_text(grid_text).is_ok() {
                tui.clipboard = Some(clipboard);
                output.messages.push("Email board copied".to_string());
            }
        }
    }
    output
}

fn run_create_puzzle_with_progress(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    tui: &mut TuiState,
    state: &mut PlayState,
) -> PlayOutput {
    let mut last_draw = Instant::now();
    let output = state.create_puzzle_with_progress(|stage, value| {
        if last_draw.elapsed() < Duration::from_millis(50) {
            return;
        }
        last_draw = Instant::now();
        let lines = progress_lines(stage, value);
        tui.overlay = OverlayState::Progress {
            title: "Create Puzzle".to_string(),
            lines,
        };
        let _ = terminal.draw(|frame| {
            let size = frame.size();
            if let OverlayState::Progress { title, lines } = &tui.overlay {
                tui.render_lines_overlay(frame, title, lines, size);
            }
        });

        if event::poll(Duration::from_millis(0)).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && matches!(key.code, KeyCode::Char('c'))
                {
                    crate::io::Cancelable::new().cancel();
                }
            }
        }
    });

    tui.overlay = OverlayState::None;
    let mut output = output;
    if output.show_board {
        let clues = state.current().solved_count();
        output.messages.clear();
        output
            .messages
            .push(format!("Created puzzle with {} clues", clues));
        output.overlay = None;
    } else if !output.messages.is_empty() {
        let message = clean_line(output.messages[0].clone());
        output.messages = vec![message];
        output.overlay = None;
    }
    output
}

fn progress_lines(stage: ProgressStage, value: usize) -> Vec<String> {
    let label = match stage {
        ProgressStage::Generate => "Generating solution",
        ProgressStage::Find => "Finding clues",
    };
    let bar = progress_bar(value, 81);
    vec![label.to_string(), bar]
}

fn progress_bar(value: usize, total: usize) -> String {
    let filled = value.min(total);
    let empty = total.saturating_sub(filled);
    format!("{}{}", "#".repeat(filled), "-".repeat(empty))
}

fn strip_ansi_lines(lines: Vec<String>) -> Vec<String> {
    lines.into_iter().map(|line| strip_ansi(&line)).collect()
}

fn digit_complete(board: &crate::puzzle::Board, digit: Digit) -> bool {
    board.solved_with(digit).len() == 9
}

fn gray_if_complete(board: &crate::puzzle::Board, digit: Digit, grid: Vec<String>) -> Vec<String> {
    if digit_complete(board, digit) {
        gray_lines(grid)
    } else {
        grid
    }
}

fn gray_lines(lines: Vec<String>) -> Vec<String> {
    lines.into_iter().map(|line| gray_line(&line)).collect()
}

fn gray_line(line: &str) -> String {
    let mut trailing = 0usize;
    for ch in line.chars().rev() {
        if ch == ' ' {
            trailing += 1;
        } else {
            break;
        }
    }
    if trailing == 0 {
        return line.bright_black().to_string();
    }
    let total = line.chars().count();
    let trimmed: String = line.chars().take(total - trailing).collect();
    if trimmed.is_empty() {
        return line.to_string();
    }
    format!("{}{}", trimmed.bright_black(), " ".repeat(trailing))
}

fn strip_ansi(input: &str) -> String {
    let mut out = String::new();
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if matches!(chars.peek(), Some('[')) {
                chars.next();
                while let Some(c) = chars.next() {
                    if c == 'm' {
                        break;
                    }
                }
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn visible_len(input: &str) -> usize {
    strip_ansi(input).chars().count()
}

fn pad_ansi_line(input: &str, width: usize) -> String {
    let len = visible_len(input);
    if len >= width {
        input.to_string()
    } else {
        format!("{}{}", input, " ".repeat(width - len))
    }
}

fn row_spacing(grid_height: usize, available_height: usize) -> usize {
    let _ = (grid_height, available_height);
    1
}

fn choose_column_spacing(available_width: u16, base_width: usize) -> usize {
    let spacing_two = base_width + 4;
    if available_width as usize >= spacing_two {
        2
    } else {
        1
    }
}

fn insert_row_gaps(lines: Vec<String>, extra: usize) -> Vec<String> {
    if extra == 0 {
        return lines;
    }
    let mut out = Vec::new();
    let mut separators = 0usize;
    for line in lines {
        let is_separator = line.trim().is_empty();
        out.push(line);
        if is_separator && separators < 2 {
            for _ in 0..extra {
                out.push(String::new());
            }
            separators += 1;
        }
    }
    out
}

fn trim_ansi_trailing_spaces(input: &str) -> String {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && matches!(chars.peek(), Some('[')) {
            let mut seq = String::from("\u{1b}");
            seq.push(chars.next().unwrap());
            while let Some(c) = chars.next() {
                seq.push(c);
                if c == 'm' {
                    break;
                }
            }
            tokens.push((seq, true));
        } else {
            tokens.push((ch.to_string(), false));
        }
    }

    while let Some((token, is_ansi)) = tokens.last() {
        if *is_ansi {
            break;
        }
        if token == " " {
            tokens.pop();
        } else {
            break;
        }
    }

    tokens.into_iter().map(|(t, _)| t).collect()
}

fn ansi_lines_to_text(lines: &[String]) -> Text<'static> {
    let mut out = Vec::new();
    for line in lines {
        out.push(ansi_line_to_line(line));
    }
    Text::from(out)
}

fn ansi_line_to_line(line: &str) -> Line<'static> {
    let mut spans = Vec::new();
    let mut style = Style::default();
    let mut buf = String::new();
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && matches!(chars.peek(), Some('[')) {
            chars.next();
            let mut code = String::new();
            let mut codes = Vec::new();
            while let Some(c) = chars.next() {
                if c == 'm' {
                    if !code.is_empty() {
                        codes.push(code.clone());
                    }
                    break;
                } else if c == ';' {
                    if !code.is_empty() {
                        codes.push(code.clone());
                        code.clear();
                    }
                } else {
                    code.push(c);
                }
            }

            if !buf.is_empty() {
                spans.push(Span::styled(buf.clone(), style));
                buf.clear();
            }

            for c in codes {
                match c.as_str() {
                    "0" => style = Style::default(),
                    "1" => style = style.add_modifier(ratatui::style::Modifier::BOLD),
                    "5" => style = style.add_modifier(ratatui::style::Modifier::SLOW_BLINK),
                    "90" => style = style.fg(Color::DarkGray),
                    "91" => style = style.fg(Color::LightRed),
                    "92" => style = style.fg(Color::LightGreen),
                    "93" => style = style.fg(Color::LightYellow),
                    "94" => style = style.fg(Color::LightBlue),
                    "95" => style = style.fg(Color::LightMagenta),
                    "96" => style = style.fg(Color::LightCyan),
                    _ => {}
                }
            }
        } else {
            buf.push(ch);
        }
    }

    if !buf.is_empty() {
        spans.push(Span::styled(buf, style));
    }
    Line::from(spans)
}

fn centered_rect(percent_x: u16, percent_y: u16, rect: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(rect);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn init_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    Terminal::new(backend)
}

fn restore_terminal(mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
