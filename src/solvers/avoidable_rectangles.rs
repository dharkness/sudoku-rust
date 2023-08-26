use super::*;

// Type 1
// .5....... .6.5.42.. ..8.71... 4....36.8 ......... 89.1..7.. 3........ ...2.7.1. .72.3..9.
//
// TODO Add types 2 and 3
// http://sudopedia.enjoysudoku.com/Avoidable_Rectangle.html
pub fn find_avoidable_rectangles(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    let candidates = board.solved();
    Rectangle::iter()
        .map(|r| (r, r.cells - candidates))
        .filter_map(|(r, cs)| cs.as_single().map(|c| (r.with_origin(c), c)))
        .filter(|(r, _)| board.value(r.top_right) == board.value(r.bottom_left))
        .filter_map(|(r, c)| board.value(r.bottom_right).known().map(|k| (c, k)))
        .filter(|(c, k)| board.candidates(*c).has(*k))
        .for_each(|(c, k)| effects.add_erase(Strategy::AvoidableRectangle, c, k));

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}
