use itertools::Itertools;

use super::*;

pub fn find_death_blossoms(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();
    let als_sets = find_als_sets(board);

    for stem in Cell::iter().filter(|cell| board.candidates(*cell).len() >= 2) {
        let stem_digits = board.candidates(stem);
        let stem_peers = stem.peers();

        let mut petals = Vec::new();
        for (index, als) in als_sets.iter().enumerate() {
            if als.cells.has(stem) {
                continue;
            }

            let mut rcc = DigitSet::empty();
            for digit in stem_digits {
                if !als.digits.has(digit) {
                    continue;
                }
                if stem_peers.has_all(als.digit_cells[digit.usize()]) {
                    rcc += digit;
                }
            }

            if !rcc.is_empty() {
                petals.push(Petal { index, rcc });
            }
        }

        if petals.len() < 2 {
            continue;
        }

        for (first, second) in petals.iter().tuple_combinations() {
            let cover = first.rcc | second.rcc;
            if !stem_digits.is_subset_of(cover) {
                continue;
            }

            let als_a = &als_sets[first.index];
            let als_b = &als_sets[second.index];
            let shared_z = (als_a.digits & als_b.digits) - stem_digits;
            if shared_z.is_empty() {
                continue;
            }

            for z in shared_z {
                let z_cells_a = als_a.digit_cells[z.usize()];
                let z_cells_b = als_b.digit_cells[z.usize()];
                if z_cells_a.is_empty() || z_cells_b.is_empty() {
                    continue;
                }

                let see_all_a = common_peers(z_cells_a);
                let see_all_b = common_peers(z_cells_b);
                let erase = (see_all_a & see_all_b & board.candidate_cells(z))
                    - als_a.cells
                    - als_b.cells
                    - stem;
                if erase.is_empty() {
                    continue;
                }

                let mut action = Action::new(Strategy::DeathBlossom);
                action.erase_cells(erase, z);
                action.clue_cell_for_digits(Verdict::Primary, stem, stem_digits);
                action.clue_cells_for_digits(Verdict::Secondary, als_a.cells, als_a.digits);
                action.clue_cells_for_digits(Verdict::Secondary, als_b.cells, als_b.digits);

                if effects.add_action(action) && single {
                    return Some(effects);
                }
            }
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

#[derive(Clone, Copy)]
struct Petal {
    index: usize,
    rcc: DigitSet,
}

#[derive(Clone)]
struct Als {
    cells: CellSet,
    digits: DigitSet,
    digit_cells: [CellSet; 9],
}

fn find_als_sets(board: &Board) -> Vec<Als> {
    let mut als_sets = Vec::new();

    for house in House::iter() {
        let cells: Vec<Cell> = house
            .cells()
            .iter()
            .filter(|cell| board.candidates(*cell).len() >= 2)
            .collect();
        let candidates: Vec<DigitSet> = cells.iter().map(|cell| board.candidates(*cell)).collect();
        let cell_count = cells.len();
        if cell_count == 0 {
            continue;
        }

        let total_masks = 1usize << cell_count;
        for mask in 1..total_masks {
            let size = mask.count_ones() as usize;
            if size == 0 {
                continue;
            }

            let mut digits = DigitSet::empty();
            for (index, cell_digits) in candidates.iter().enumerate() {
                if (mask & (1usize << index)) != 0 {
                    digits |= *cell_digits;
                }
            }

            if digits.len() != size + 1 {
                continue;
            }

            let mut subset_cells = CellSet::empty();
            let mut digit_cells = [CellSet::empty(); 9];
            for (index, cell_digits) in candidates.iter().enumerate() {
                if (mask & (1usize << index)) != 0 {
                    let cell = cells[index];
                    subset_cells.add(cell);
                    for digit in *cell_digits {
                        digit_cells[digit.usize()].add(cell);
                    }
                }
            }

            als_sets.push(Als {
                cells: subset_cells,
                digits,
                digit_cells,
            });
        }
    }

    als_sets
}

fn common_peers(cells: CellSet) -> CellSet {
    let mut common = CellSet::full();
    if cells.is_empty() {
        return CellSet::empty();
    }
    for cell in cells {
        common &= cell.peers();
        if common.is_empty() {
            break;
        }
    }
    common
}

#[cfg(test)]
mod tests {
    use crate::io::{Parse, Parser};
    use crate::puzzle::Options;
    use crate::*;

    use super::*;

    #[test]
    fn example_1() {
        let board = Parse::packed_with_options(Options::errors()).parse_simple(
            "
                8..4.9...
                2..07....
                .5.2..3..
                .428.1.3.
                .3.....9.
                .8.9.321.
                ..6..7.4.
                ....2..1.
                ...6.4..3
            ",
        );

        let got = find_death_blossoms(&board, false).expect("not found");
        assert!(got.action_count() > 0);
    }

    #[test]
    fn example_2() {
        let parser = Parse::packed_with_options(Options::errors());
        let (board, effects, failed) = parser.parse(
            "163578040874000135952040786080692010020000000609050820030015000098000401000000070",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_death_blossoms(&board, false).expect("not found");
        assert_erase!(got, D9, 3);
        assert_erase!(got, E9, 3);
        assert_erase!(got, F9, 3);
        assert_erase!(got, J7, 3);
    }

    #[test]
    fn massive_example() {
        let parser = Parse::packed_with_options(Options::errors());
        let (board, effects, failed) = parser.parse(
            "050000060094200500000003001630020009000000700000009040203000000060041008005600090",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_death_blossoms(&board, false).expect("not found");
        assert!(got.action_count() > 0);
    }

    #[test]
    fn six_eliminations_example() {
        let parser = Parse::packed_with_options(Options::errors());
        let (board, effects, failed) = parser.parse(
            "010000004000060000400002030000005200090080050001020007009000060300100800020803400",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_death_blossoms(&board, false).expect("not found");
        assert!(got.action_count() > 0);
    }

    #[test]
    fn one_box_example() {
        let parser = Parse::packed_with_options(Options::errors());
        let (board, effects, failed) = parser.parse(
            "000020000009087002300501000000010000900802360000900075050000010067004003200100007",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_death_blossoms(&board, false).expect("not found");
        assert!(got.action_count() > 0);
    }
}
