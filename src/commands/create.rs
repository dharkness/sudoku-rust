use clap::Args;

use crate::build::Generator;
use crate::io::{print_candidates, print_values, Cancelable};
use crate::puzzle::{Options, Player};

#[derive(Debug, Args)]
pub struct CreateArgs {
    /// Shuffles the cells before generating
    #[clap(short, long)]
    shuffle: bool,

    /// Identifies the cells that will receive starting clues
    #[clap(short, long)]
    pattern: Option<String>,
}

/// Creates a new puzzle and prints it to stdout.
pub fn create_puzzle(args: CreateArgs, cancelable: &Cancelable) {
    let player = Player::new(Options::all());
    let mut generator = Generator::new(args.shuffle);

    match generator.generate(&player, cancelable) {
        Some(board) => {
            print_values(&board);
            if !board.is_solved() {
                print_candidates(&board);
            }
            println!("Board: {}", board);
        }
        None => {
            println!("Failed to create a puzzle.");
        }
    }
}
