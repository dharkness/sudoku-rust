pub use bingo::{bingo, BingoArgs};
pub use create::{create_puzzle, CreateArgs};
pub use extract::{extract_patterns, ExtractArgs};
pub use find::{find_solutions, FindArgs};
pub use play::{start_player, PlayArgs};
pub use profile::{profile_puzzles, ProfileArgs};
pub use solve::{solve_puzzles, SolveArgs};
pub use tui::{start_tui, TuiArgs};

mod bingo;
mod create;
mod extract;
mod find;
mod play;
mod play_core;
mod profile;
mod solve;
mod tui;
