//! Provides a text-based interface for creating and playing Sudoku puzzles.

use std::io::{stdout, Write};

use clap::Args;

use crate::io::{
    print_all_and_single_candidates, print_all_and_single_candidates_with_highlight,
    print_solved_values,
};
use crate::symbols::MISSING;

use super::play_core::{
    parse_command_line, NewPuzzleInput, PlayCommand, PlayOptionsArgs, PlayOutput, PlayState,
};

#[derive(Debug, Args)]
#[clap(disable_help_flag = true)]
pub struct PlayArgs {
    /// Print help information
    #[clap(long, action = clap::ArgAction::HelpLong)]
    help: Option<bool>,

    #[clap(flatten)]
    options: PlayOptionsArgs,

    /// Clues for a starting puzzle
    puzzle: Option<String>,
}

impl PlayArgs {
    pub fn new() -> Self {
        Self {
            help: None,
            options: PlayOptionsArgs::default(),
            puzzle: None,
        }
    }
}

pub fn start_player(args: PlayArgs) {
    let (mut state, init_output) = PlayState::new(args.options.options(), args.puzzle.clone());
    let mut show_board = init_output.show_board;
    emit_output(init_output, &mut show_board);
    if args.puzzle.is_none() {
        let output = state.apply(PlayCommand::CreatePuzzle);
        emit_output(output, &mut show_board);
    }

    loop {
        let board = state.current();
        if show_board {
            show_board = false;
            if board.is_fully_solved() {
                println!("\n==> Congratulations!\n");
                print_solved_values(board);
                println!();
            } else if let Some(action) = state.highlight() {
                print_all_and_single_candidates_with_highlight(board, action);
                println!();
            } else {
                print_all_and_single_candidates(board);
                println!();
            }
        }

        print!(
            "[ {} solved - {} unsolved ] ",
            board.solved_count(),
            board.unsolved_count()
        );
        let _ = stdout().flush();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_uppercase();
        if input.is_empty() {
            continue;
        }
        let input = input.split(' ').collect::<Vec<_>>();

        if input[0] == "N" {
            if let Some(puzzle) = prompt_new_puzzle() {
                let output = state.apply(PlayCommand::NewPuzzle {
                    input: NewPuzzleInput::Puzzle,
                    puzzle: Some(puzzle),
                });
                emit_output(output, &mut show_board);
            }
            continue;
        }

        match parse_command_line(&input.join(" ")) {
            Ok(command) => {
                let output = state.apply(command);
                if output.quit {
                    break;
                }
                emit_output(output, &mut show_board);
            }
            Err(error) => {
                if !error.to_string().is_empty() {
                    println!("\n==> {}\n", error);
                }
            }
        }
    }
}

fn prompt_new_puzzle() -> Option<String> {
    println!(concat!(
        "\n==> Enter the givens\n\n",
        "  - enter up to 81 digits\n",
        "  - use period or zero to leave a cell blank\n",
        "  - spaces are ignored\n",
        "  - leave empty to cancel\n",
        "  - enter 'E' for an empty puzzle\n",
    ));

    loop {
        print!("> ");
        let _ = stdout().flush();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().replace(' ', "").replace(MISSING, ".");
        if input.is_empty() {
            println!();
            return None;
        }
        if input.len() >= 160 || input.len() <= 81 {
            return Some(input);
        }

        println!(
            concat!(
                "\n==> Expected 81 or 162 digits, got {}\n\n",
                "{}\n",
                "        |        |        |        |        |        |        |        |        |\n",
            ),
            input.len(),
            input
        );
    }
}

fn emit_output(output: PlayOutput, show_board: &mut bool) {
    if output.show_board {
        *show_board = true;
    }

    for message in output.messages {
        println!("{}", message);
    }

    if let Some(overlay) = output.overlay {
        match overlay {
            super::play_core::Overlay::Help(text) => println!("\n{}", text),
            super::play_core::Overlay::Board { lines, .. } => {
                for line in lines {
                    println!("{}", line);
                }
            }
            super::play_core::Overlay::Text { lines, .. } => {
                for line in lines {
                    println!("{}", line);
                }
            }
        }
    }
}
