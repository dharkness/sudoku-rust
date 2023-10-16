use itertools::Itertools;

pub fn strip_leading_whitespace(s: &str) -> String {
    s.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .join("\n")
}
