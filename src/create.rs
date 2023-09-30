use clap::Args;

use crate::build::Generator;
use crate::io::{print_candidates, print_values, Cancelable};

#[derive(Debug, Args)]
pub struct CreateArgs {
    /// Shuffles the cells before generating
    #[clap(short = 's', long = "shuffle")]
    shuffle: bool,

    /// Identifies the cells that will receive starting clues
    #[clap(long, short = 'p')]
    pattern: Option<String>,
}

/// Creates a new puzzle and prints it to stdout.
pub fn create(args: CreateArgs, cancelable: &Cancelable) {
    let mut generator = Generator::new(args.shuffle);

    match generator.generate(cancelable) {
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
