use std::collections::VecDeque;

use super::*;

const DIGIT_COUNT: usize = Digit::COUNT as usize;
const CELL_COUNT: usize = Cell::COUNT as usize;
const CANDIDATE_COUNT: usize = CELL_COUNT * DIGIT_COUNT;

pub fn find_three_d_medusa(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    let (present, strong_adj) = build_strong_links(board);
    let components = build_components(&present, &strong_adj);

    for component in components {
        if component.color_nodes[0].is_empty() || component.color_nodes[1].is_empty() {
            continue;
        }

        if add_rule_1(board, &component, single, &mut effects) {
            return Some(effects);
        }
        if add_rule_2(board, &component, single, &mut effects) {
            return Some(effects);
        }
        if add_rule_3(board, &component, single, &mut effects) {
            return Some(effects);
        }
        if add_rule_4(board, &component, single, &mut effects) {
            return Some(effects);
        }
        if add_rule_6(board, &component, single, &mut effects) {
            return Some(effects);
        }
        if add_rule_5(board, &component, single, &mut effects) {
            return Some(effects);
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Color {
    Primary,
    Secondary,
}

impl Color {
    fn opposite(self) -> Self {
        match self {
            Self::Primary => Self::Secondary,
            Self::Secondary => Self::Primary,
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Primary => 0,
            Self::Secondary => 1,
        }
    }
}

struct Component {
    color_nodes: [Vec<usize>; 2],
    color_digit_cells: [[CellSet; 9]; 2],
    color_cell_digits: [[DigitSet; 81]; 2],
}

fn build_strong_links(board: &Board) -> (Vec<bool>, Vec<Vec<usize>>) {
    let mut present = vec![false; CANDIDATE_COUNT];
    let mut strong_adj = vec![Vec::new(); CANDIDATE_COUNT];

    for cell in Cell::iter() {
        let candidates = board.candidates(cell);
        for digit in candidates {
            let index = candidate_index(cell, digit);
            present[index] = true;
        }

        if let Some((first, second)) = candidates.as_pair() {
            let a = candidate_index(cell, first);
            let b = candidate_index(cell, second);
            add_strong_link(&mut strong_adj, a, b);
        }
    }

    for digit in Digit::iter() {
        for house in House::iter() {
            let cells = board.house_candidate_cells(house, digit);
            if let Some((first, second)) = cells.as_pair() {
                let a = candidate_index(first, digit);
                let b = candidate_index(second, digit);
                add_strong_link(&mut strong_adj, a, b);
            }
        }
    }

    (present, strong_adj)
}

fn add_strong_link(adj: &mut [Vec<usize>], a: usize, b: usize) {
    adj[a].push(b);
    adj[b].push(a);
}

fn build_components(present: &[bool], strong_adj: &[Vec<usize>]) -> Vec<Component> {
    let mut colors: Vec<Option<Color>> = vec![None; CANDIDATE_COUNT];
    let mut components = Vec::new();

    for start in 0..CANDIDATE_COUNT {
        if !present[start] || colors[start].is_some() {
            continue;
        }

        let mut nodes = Vec::new();
        let mut queue = VecDeque::new();
        colors[start] = Some(Color::Primary);
        queue.push_back(start);

        while let Some(current) = queue.pop_front() {
            nodes.push(current);
            let color = colors[current].unwrap();
            for &neighbor in &strong_adj[current] {
                if !present[neighbor] {
                    continue;
                }
                match colors[neighbor] {
                    None => {
                        colors[neighbor] = Some(color.opposite());
                        queue.push_back(neighbor);
                    }
                    Some(existing) => {
                        if existing == color {
                            // Odd loop in strong links: ignore for now.
                        }
                    }
                }
            }
        }

        components.push(build_component(&nodes, &colors));
    }

    components
}

fn build_component(nodes: &[usize], colors: &[Option<Color>]) -> Component {
    let mut color_nodes = [Vec::new(), Vec::new()];
    let mut color_digit_cells = [[CellSet::empty(); 9]; 2];
    let mut color_cell_digits = [[DigitSet::empty(); 81]; 2];

    for &node in nodes {
        let Some(color) = colors[node] else {
            continue;
        };
        let index = color.index();
        let cell = candidate_cell(node);
        let digit = candidate_digit(node);

        color_nodes[index].push(node);
        color_digit_cells[index][digit.usize()].add(cell);
        color_cell_digits[index][cell.usize()] += digit;
    }

    Component {
        color_nodes,
        color_digit_cells,
        color_cell_digits,
    }
}

fn candidate_index(cell: Cell, digit: Digit) -> usize {
    cell.usize() * DIGIT_COUNT + digit.usize()
}

fn candidate_cell(index: usize) -> Cell {
    Cell::new((index / DIGIT_COUNT) as u8)
}

fn candidate_digit(index: usize) -> Digit {
    Digit::new((index % DIGIT_COUNT) as u32)
}

fn add_rule_1(board: &Board, component: &Component, single: bool, effects: &mut Effects) -> bool {
    for (color_index, digits_by_cell) in component.color_cell_digits.iter().enumerate() {
        for digits in digits_by_cell.iter() {
            if digits.len() < 2 {
                continue;
            }
            let color = match color_index {
                0 => Color::Primary,
                _ => Color::Secondary,
            };
            if let Some(action) = eliminate_color(board, component, color) {
                effects.add_action(action);
                return single;
            }
        }
    }

    false
}

fn add_rule_2(board: &Board, component: &Component, single: bool, effects: &mut Effects) -> bool {
    for color_index in 0..2 {
        for digit in Digit::iter() {
            let cells = component.color_digit_cells[color_index][digit.usize()];
            if cells.len() < 2 {
                continue;
            }
            for house in House::iter() {
                if (cells & house.cells()).len() >= 2 {
                    let color = match color_index {
                        0 => Color::Primary,
                        _ => Color::Secondary,
                    };
                    if let Some(action) = eliminate_color(board, component, color) {
                        effects.add_action(action);
                        return single;
                    }
                }
            }
        }
    }

    false
}

fn add_rule_3(board: &Board, component: &Component, single: bool, effects: &mut Effects) -> bool {
    let mut action = Action::new(Strategy::ThreeDMedusa);

    for cell in Cell::iter() {
        let primary = component.color_cell_digits[Color::Primary.index()][cell.usize()];
        let secondary = component.color_cell_digits[Color::Secondary.index()][cell.usize()];
        if primary.is_empty() || secondary.is_empty() {
            continue;
        }
        let uncolored = board.candidates(cell) - (primary | secondary);
        if !uncolored.is_empty() {
            action.erase_digits(cell, uncolored);
        }
    }

    if action.is_empty() {
        return false;
    }

    add_component_clues(&mut action, component);
    effects.add_action(action) && single
}

fn add_rule_4(board: &Board, component: &Component, single: bool, effects: &mut Effects) -> bool {
    let mut action = Action::new(Strategy::ThreeDMedusa);

    for cell in Cell::iter() {
        let primary = component.color_cell_digits[Color::Primary.index()][cell.usize()];
        let secondary = component.color_cell_digits[Color::Secondary.index()][cell.usize()];
        let uncolored = board.candidates(cell) - (primary | secondary);
        if uncolored.is_empty() {
            continue;
        }
        for digit in uncolored {
            if sees_color(component, cell, digit, Color::Primary)
                && sees_color(component, cell, digit, Color::Secondary)
            {
                action.erase(cell, digit);
            }
        }
    }

    if action.is_empty() {
        return false;
    }

    add_component_clues(&mut action, component);
    effects.add_action(action) && single
}

fn add_rule_5(board: &Board, component: &Component, single: bool, effects: &mut Effects) -> bool {
    let mut action = Action::new(Strategy::ThreeDMedusa);

    for cell in Cell::iter() {
        let primary = component.color_cell_digits[Color::Primary.index()][cell.usize()];
        let secondary = component.color_cell_digits[Color::Secondary.index()][cell.usize()];
        let uncolored = board.candidates(cell) - (primary | secondary);
        if uncolored.is_empty() {
            continue;
        }

        let has_primary = !primary.is_empty();
        let has_secondary = !secondary.is_empty();

        for digit in uncolored {
            let sees_primary = sees_color(component, cell, digit, Color::Primary);
            let sees_secondary = sees_color(component, cell, digit, Color::Secondary);
            if (sees_primary && has_secondary) || (sees_secondary && has_primary) {
                action.erase(cell, digit);
            }
        }
    }

    if action.is_empty() {
        return false;
    }

    add_component_clues(&mut action, component);
    effects.add_action(action) && single
}

fn add_rule_6(board: &Board, component: &Component, single: bool, effects: &mut Effects) -> bool {
    for cell in Cell::iter() {
        let primary = component.color_cell_digits[Color::Primary.index()][cell.usize()];
        let secondary = component.color_cell_digits[Color::Secondary.index()][cell.usize()];
        if !primary.is_empty() || !secondary.is_empty() {
            continue;
        }

        let digits = board.candidates(cell);
        if digits.is_empty() {
            continue;
        }

        let mut all_primary = true;
        let mut all_secondary = true;

        for digit in digits {
            if !sees_color(component, cell, digit, Color::Primary) {
                all_primary = false;
            }
            if !sees_color(component, cell, digit, Color::Secondary) {
                all_secondary = false;
            }
        }

        if all_primary && !all_secondary {
            if let Some(action) = eliminate_color(board, component, Color::Primary) {
                effects.add_action(action);
                return single;
            }
        }

        if all_secondary && !all_primary {
            if let Some(action) = eliminate_color(board, component, Color::Secondary) {
                effects.add_action(action);
                return single;
            }
        }
    }

    false
}

fn sees_color(component: &Component, cell: Cell, digit: Digit, color: Color) -> bool {
    let cells = component.color_digit_cells[color.index()][digit.usize()];
    !(cell.peers() & cells).is_empty()
}

fn eliminate_color(board: &Board, component: &Component, color: Color) -> Option<Action> {
    let mut action = Action::new(Strategy::ThreeDMedusa);
    for &node in &component.color_nodes[color.index()] {
        let cell = candidate_cell(node);
        let digit = candidate_digit(node);
        if board.is_candidate(cell, digit) {
            action.erase(cell, digit);
        }
    }

    if action.is_empty() {
        return None;
    }

    add_component_clues(&mut action, component);
    Some(action)
}

fn add_component_clues(action: &mut Action, component: &Component) {
    for &node in &component.color_nodes[Color::Primary.index()] {
        let cell = candidate_cell(node);
        let digit = candidate_digit(node);
        action.clue_cell_for_digit(Verdict::Primary, cell, digit);
    }
    for &node in &component.color_nodes[Color::Secondary.index()] {
        let cell = candidate_cell(node);
        let digit = candidate_digit(node);
        action.clue_cell_for_digit(Verdict::Secondary, cell, digit);
    }
}

#[cfg(test)]
mod tests {
    use crate::io::{Parse, Parser};
    use crate::puzzle::Options;
    use crate::*;

    use super::*;

    fn assert_any_erase(actions: &[Action], cell: Cell, digit: Digit) {
        assert!(
            actions.iter().any(|action| action.erases(cell, digit)),
            "expected erase of {} from {}",
            digit,
            cell
        );
    }

    #[test]
    fn rule_1_example() {
        let parser = Parse::packed_with_options(Options::all());
        let (board, effects, failed) = parser.parse(
            "093824560085600002206075008321769845000258300578040296850016723007082650002507180",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_three_d_medusa(&board, false).expect("not found");
        assert_any_erase(got.actions(), cell!(H2), digit!(1));
    }

    #[test]
    fn rule_2_example() {
        let parser = Parse::packed_with_options(Options::all());
        let (board, effects, failed) = parser.parse(
            "300052000250300010004607523093200805570000030408035060005408300030506084840023056",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_three_d_medusa(&board, false).expect("not found");
        assert_any_erase(got.actions(), cell!(E5), digit!(4));
    }

    #[test]
    fn rule_3_example() {
        let parser = Parse::packed_with_options(Options::all());
        let (board, effects, failed) = parser.parse(
            "290000830000020970000109402845761293600000547009045008903407000060030709050000384",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_three_d_medusa(&board, false).expect("not found");
        assert_any_erase(got.actions(), cell!(C2), digit!(8));
    }

    #[test]
    fn rule_4_example() {
        let parser = Parse::packed_with_options(Options::none());
        let (board, effects, failed) = parser.parse(
            "100056003043090000800043002030560210950421037021030000317980005000310970000670301",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_three_d_medusa(&board, false).expect("not found");
        assert_any_erase(got.actions(), cell!(B1), digit!(7));
    }

    #[test]
    fn rule_5_example() {
        let parser = Parse::packed_with_options(Options::all());
        let (board, effects, failed) = parser.parse(
            "923407015876050924500200030769020140432000059185004260098042071207030486000708092",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_three_d_medusa(&board, false).expect("not found");
        assert_any_erase(got.actions(), cell!(E5), digit!(1));
    }

    #[test]
    fn rule_6_example() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B0f4m4j04b77q4i0g02440d09440g0501030f4p4o0g45060o0d094i7t070l0f05087o0d7r4o4o067n7n0d074k4mbn4i0d03020gbm06bn4k094k4k040f0c0a0g0d010307b67o064k82070f4kbo0c01b84404",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_three_d_medusa(&board, false).expect("not found");
        assert_any_erase(got.actions(), cell!(A3), digit!(1));
    }

    #[test]
    fn rule_6_three_candidates_example() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B0912080n020v16070606100sba8607014aba01070uc6968q8202ba5u4405041i1i2c09010309010708021m1m0504062c7n7n0508032c5u042e1l1j538i057o0546067s7y7y2c015u02010905074y034y04",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_three_d_medusa(&board, false).expect("not found");
        assert_any_erase(got.actions(), cell!(A6), digit!(4));
    }
}
