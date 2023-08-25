use super::*;

// Type 1
// .5....... .6.5.42.. ..8.71... 4....36.8 ......... 89.1..7.. 3........ ...2.7.1. .72.3..9.
//
// TODO add types 2 and 3 by looping over all possible two-block rectangles
// http://sudopedia.enjoysudoku.com/Avoidable_Rectangle.html
pub fn find_avoidable_rectangles(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    let candidates = board.knowns() - board.givens();
    if candidates.size() < 3 {
        return None;
    }

    for rectangle in candidates
        .iter()
        .combinations(3)
        .map(Rectangle::try_from)
        .filter_map(Result::ok)
        .filter(|r| r.block_count() == 2)
    {
        let top_left = rectangle.top_left();
        let top_right = rectangle.top_right();
        let bottom_left = rectangle.bottom_left();
        let bottom_right = rectangle.bottom_right();

        let top_left_value = board.value(top_left);
        let top_right_value = board.value(top_right);
        let bottom_left_value = board.value(bottom_left);
        let bottom_right_value = board.value(bottom_right);

        for (known, cell) in [
            (
                top_left_value,
                bottom_right_value,
                top_right_value,
                bottom_left,
            ),
            (
                top_left_value,
                bottom_right_value,
                bottom_left_value,
                top_right,
            ),
            (
                top_right_value,
                bottom_left_value,
                top_left_value,
                bottom_right,
            ),
            (
                top_right_value,
                bottom_left_value,
                bottom_right_value,
                top_left,
            ),
        ]
        .iter()
        .filter(|(pair1, pair2, _, _)| pair1 == pair2)
        .filter_map(|(_, _, single, unsolved)| single.known().map(|known| (known, *unsolved)))
        .filter(|(known, cell)| board.candidates(*cell).has(*known))
        {
            effects.add_erase(Strategy::AvoidableRectangle, cell, known);
        }
        // if top_left_value == bottom_right_value {
        //     if let Some(known) = top_right_value.known() {
        //         if board.candidates(bottom_left).has(known) {
        //             effects.add_erase(Strategy::AvoidableRectangle, bottom_left, known);
        //         }
        //     } else if let Some(known) = bottom_left_value.known() {
        //         if board.candidates(top_right).has(known) {
        //             effects.add_erase(Strategy::AvoidableRectangle, top_right, known);
        //         }
        //     }
        // } else if top_right_value == bottom_left_value {
        //     if let Some(known) = top_left_value.known() {
        //         if board.candidates(bottom_right).has(known) {
        //             effects.add_erase(Strategy::AvoidableRectangle, bottom_right, known);
        //         }
        //     } else if let Some(known) = bottom_right_value.known() {
        //         if board.candidates(top_left).has(known) {
        //             effects.add_erase(Strategy::AvoidableRectangle, top_left, known);
        //         }
        //     }
        // }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}
