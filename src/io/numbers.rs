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

/// Returns the ordinal suffix for a given number (e.g., "st" for 1, "nd" for 2).
pub const fn ordinal_suffix(n: usize) -> &'static str {
    // Handle special cases for 11th, 12th, 13th, 121st, 122nd, 123rd, etc.
    if n % 100 >= 11 && n % 100 <= 13 {
        return "th";
    }

    match n % 10 {
        1 => "st",
        2 => "nd",
        3 => "rd",
        _ => "th",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ordinal_suffix_basic() {
        assert_eq!("st", ordinal_suffix(1));
        assert_eq!("nd", ordinal_suffix(2));
        assert_eq!("rd", ordinal_suffix(3));
        assert_eq!("th", ordinal_suffix(4));
        assert_eq!("th", ordinal_suffix(5));
        assert_eq!("th", ordinal_suffix(6));
        assert_eq!("th", ordinal_suffix(7));
        assert_eq!("th", ordinal_suffix(8));
        assert_eq!("th", ordinal_suffix(9));
        assert_eq!("th", ordinal_suffix(10));
    }

    #[test]
    fn test_ordinal_suffix_teens() {
        // The special cases: 11th, 12th, 13th (not 11st, 12nd, 13rd)
        assert_eq!("th", ordinal_suffix(11));
        assert_eq!("th", ordinal_suffix(12));
        assert_eq!("th", ordinal_suffix(13));
    }

    #[test]
    fn test_ordinal_suffix_twenties() {
        assert_eq!("th", ordinal_suffix(20));
        assert_eq!("st", ordinal_suffix(21));
        assert_eq!("nd", ordinal_suffix(22));
        assert_eq!("rd", ordinal_suffix(23));
        assert_eq!("th", ordinal_suffix(24));
    }

    #[test]
    fn test_ordinal_suffix_hundreds() {
        assert_eq!("st", ordinal_suffix(101));
        assert_eq!("nd", ordinal_suffix(102));
        assert_eq!("rd", ordinal_suffix(103));
        assert_eq!("th", ordinal_suffix(111));
        assert_eq!("th", ordinal_suffix(112));
        assert_eq!("th", ordinal_suffix(113));
        assert_eq!("st", ordinal_suffix(121));
    }

    #[test]
    fn test_ordinal_suffix_large_numbers() {
        assert_eq!("st", ordinal_suffix(1001));
        assert_eq!("th", ordinal_suffix(1011));
        assert_eq!("st", ordinal_suffix(1021));
        assert_eq!("rd", ordinal_suffix(9999993));
    }

    #[test]
    fn test_ordinal_suffix_sudoku_range() {
        // Test the range we'll actually use (1-81 for Sudoku)
        assert_eq!("st", ordinal_suffix(1));
        assert_eq!("nd", ordinal_suffix(2));
        assert_eq!("rd", ordinal_suffix(3));
        assert_eq!("th", ordinal_suffix(11));
        assert_eq!("th", ordinal_suffix(12));
        assert_eq!("th", ordinal_suffix(13));
        assert_eq!("st", ordinal_suffix(21));
        assert_eq!("nd", ordinal_suffix(22));
        assert_eq!("rd", ordinal_suffix(23));
        assert_eq!("st", ordinal_suffix(31));
        assert_eq!("st", ordinal_suffix(41));
        assert_eq!("st", ordinal_suffix(51));
        assert_eq!("st", ordinal_suffix(61));
        assert_eq!("st", ordinal_suffix(71));
        assert_eq!("st", ordinal_suffix(81));
    }
}
