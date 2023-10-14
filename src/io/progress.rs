const FILLED: &str =
    "|---------=========---------=========---------=========---------=========---------|";
const EMPTY: &str =
    "|                                                                                 |";

/// Prints a progress bar to the console.
pub fn show_progress(size: usize) {
    println!("{}{}", &FILLED[..size + 1], &EMPTY[size + 1..]);
}
