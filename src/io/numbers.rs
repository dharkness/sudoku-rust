//! Format numbers and durations.

use std::time::Duration;

/// Formats a duration in microseconds with commas.
pub fn format_runtime(runtime: Duration) -> String {
    format_number(runtime.as_micros())
}

/// Formats a number with commas as thousands separator.
pub fn format_number(number: u128) -> String {
    number
        .to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(",")
}
