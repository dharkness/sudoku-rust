use std::collections::HashMap;
use std::time::Duration;

use itertools::Itertools;

use crate::io::{format_number, format_runtime};
use crate::puzzle::Strategy;

/// Tracks the number of times a strategy was called, the number of times it found a solution,
/// and the total time spent in the strategy.
pub struct Timings {
    timings: HashMap<Strategy, HashMap<usize, (usize, Duration)>>,
    found: usize,
    duration: Duration,
}

impl Timings {
    pub fn new() -> Timings {
        Timings {
            timings: HashMap::new(),
            found: 0,
            duration: Duration::default(),
        }
    }

    pub fn add(&mut self, strategy: Strategy, found: usize, duration: Duration) {
        self.found += found;
        self.duration += duration;

        let entry = self.timings.entry(strategy).or_default();
        let (count, total) = entry.entry(found).or_default();
        *count += 1;
        *total += duration;
    }

    pub fn print_details(&self) {
        println!("Strategy              Called   Found       Total    Call Avg         Avg");
        for (strategy, found_times) in self.timings.iter().sorted_by(|(a, _), (b, _)| a.cmp(b)) {
            for (found, (count, duration)) in found_times
                .iter()
                .sorted_by(|(_, (a, _)), (_, (b, _))| a.cmp(b))
            {
                let total = *count * *found;
                println!(
                    "{:20} {:>11} {:>11} {:>11} {:>11} {:>11}",
                    strategy.label(),
                    format_number(*count as u128),
                    if *found == 0 {
                        "-".to_string()
                    } else {
                        format_number(*found as u128)
                    },
                    format_runtime(*duration),
                    format_runtime(duration.div_f64(*count as f64)),
                    if total == 0 {
                        "-".to_string()
                    } else {
                        format_runtime(duration.div_f64(total as f64))
                    }
                );
            }
        }
    }

    pub fn print_totals(&self) {
        println!(
            "Strategy                  Called       Found       Total    Call Avg         Avg"
        );
        for (strategy, (found, count, duration)) in self
            .timings
            .iter()
            .map(|(strategy, found_times)| {
                (
                    strategy,
                    found_times.iter().fold(
                        (0usize, 0usize, Duration::default()),
                        |acc, (found, (count, duration))| {
                            (acc.0 + *found * *count, acc.1 + *count, acc.2 + *duration)
                        },
                    ),
                )
            })
            .sorted_by(|(_, (_, _, a)), (_, (_, _, b))| b.cmp(a))
        {
            println!(
                "{:20} {:>11} {:>11} {:>11} {:>11} {:>11}",
                strategy.label(),
                format_number(count as u128),
                if found == 0 {
                    "-".to_string()
                } else {
                    format_number(found as u128)
                },
                format_runtime(duration),
                format_runtime(duration.div_f64(count as f64)),
                if found == 0 {
                    "-".to_string()
                } else {
                    format_runtime(duration.div_f64(found as f64))
                }
            );
        }
    }
}
