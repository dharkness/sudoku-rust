use clap::Args;
use std::process::exit;

use crate::build::{Finder, Generator};
use crate::io::{print_candidates, print_values, Cancelable, Parse};
use crate::puzzle::{Options, Player};

#[derive(Debug, Args)]
pub struct CreateArgs {
    /// Randomizes the cells before generating
    #[clap(short, long)]
    randomize: bool,

    /// Identifies the cells that will receive starting clues
    #[clap(short, long)]
    pattern: Option<String>,

    /// Provides the solution to start from
    #[clap(short, long)]
    solution: Option<String>,
}

/// Creates a new puzzle and prints it to stdout,
/// using the given solution and/or pattern if provided.
pub fn create_puzzle(args: CreateArgs, cancelable: &Cancelable) {
    let board = match args.solution {
        Some(solution) => {
            let parser = Parse::packed_with_options(Options::all());
            let (board, effects, failure) = parser.parse(&solution);

            if let Some((cell, known)) = failure {
                print_candidates(&board);
                eprintln!("\n==> Setting {} to {} will cause errors\n", cell, known);
                effects.print_errors();
                exit(1);
            }
            if !board.is_solved() {
                print_candidates(&board);
                eprintln!("\n==> You must provide a complete solution");
                exit(1);
            }

            board
        }
        None => {
            let player = Player::new(Options::all());
            let mut generator = Generator::new(args.randomize);

            match generator.generate(&player, cancelable) {
                Some(board) => {
                    if cancelable.is_canceled() {
                        print_candidates(&board);
                        println!("\n==> Puzzle generation canceled");
                        exit(1);
                    }
                    if !board.is_solved() {
                        print_candidates(&board);
                        println!("\n==> Failed to generate a complete solution");
                        exit(1);
                    }

                    board
                }
                None => {
                    println!("\n==> Failed to generate a complete solution");
                    exit(1);
                }
            }
        }
    };

    print_values(&board);
    println!("\n==> Seeking a solvable starting puzzle...");

    let mut finder = Finder::new();
    match finder.backtracking_find(board, cancelable) {
        Some(board) => {
            print_values(&board);
            println!("\n==> Puzzle created");
        }
        None => {
            println!("\n==> Failed to find a solvable starting puzzle");
            exit(1);
        }
    }
}
