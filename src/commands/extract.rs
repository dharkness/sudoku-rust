use clap::Args;
use itertools::Itertools;
use std::collections::HashMap;
use std::io::BufRead;
use std::time::Instant;

use crate::io::{format_number, format_runtime, Cancelable};
use crate::layout::CellSet;

#[derive(Debug, Args)]
pub struct ExtractArgs {
    /// Stream the patterns to STDOUT without counts
    #[clap(short, long)]
    stream: bool,

    /// Print total counts only
    #[clap(short, long)]
    total: bool,
}

/// Scans puzzles from STDIN to build a collection of starting patterns.
pub fn extract_patterns(args: ExtractArgs, cancelable: &Cancelable) {
    let stdin = std::io::stdin();

    if args.stream {
        for puzzle in stdin.lock().lines().map_while(Result::ok) {
            println!("{}", CellSet::new_from_pattern(&puzzle).pattern_string());
        }
        return;
    }

    let runtime = Instant::now();
    let mut patterns = HashMap::new();
    let mut sizes = HashMap::new();
    let mut total_size: usize = 0;
    let mut count: usize = 0;

    for puzzle in stdin.lock().lines().map_while(Result::ok) {
        let pattern = CellSet::new_from_pattern(&puzzle);

        total_size += pattern.len();
        *sizes.entry(pattern.len()).or_default() += 1;
        *patterns.entry(pattern).or_default() += 1;
        count += 1;
        if cancelable.is_canceled() {
            break;
        }
    }

    if count == 0 {
        println!("no patterns found");
        return;
    }

    let pattern_count = patterns.len();
    let size_count = sizes.len();
    if !args.total {
        for (pattern, count) in patterns.into_iter().sorted_by(|a, b| Ord::cmp(&a.1, &b.1)) {
            println!("{} - {:<2}", pattern.pattern_string(), format_number(count));
        }
        println!();
        for (size, count) in sizes.into_iter().sorted_by(|a, b| Ord::cmp(&a.1, &b.1)) {
            println!("{:<2} - {:<2}", size, format_number(count));
        }
    }

    println!(
        "\nfound {} patterns ({} unique) with {:.2} average size ({} unique) in {} µs",
        format_number(count as u128),
        format_number(pattern_count as u128),
        total_size as f32 / count as f32,
        format_number(size_count as u128),
        format_runtime(runtime.elapsed())
    );
}
