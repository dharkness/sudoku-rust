use std::collections::HashMap;

use super::*;

pub fn find_chute_remote_pairs(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    let mut pairs_by_candidates: HashMap<DigitSet, CellSet> = HashMap::new();
    for (cell, candidates) in board.cells_with_n_candidates_iter(2) {
        *pairs_by_candidates.entry(candidates).or_default() += cell;
    }

    for (pair, cells) in pairs_by_candidates {
        let Some((d1, d2)) = pair.as_pair() else {
            continue;
        };

        if add_chute_remote_pairs(board, pair, d1, d2, cells, &mut effects, single) {
            return Some(effects);
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

fn add_chute_remote_pairs(
    board: &Board,
    pair: DigitSet,
    d1: Digit,
    d2: Digit,
    cells: CellSet,
    effects: &mut Effects,
    single: bool,
) -> bool {
    for (c1, c2) in cells.pair_iter() {
        if c1.sees(c2) {
            continue;
        }

        let Some(chute) = chute_info(c1, c2) else {
            continue;
        };

        if add_chute_remote_pair_action(board, pair, d1, d2, c1, c2, chute, effects, single) {
            return true;
        }
    }

    false
}

fn add_chute_remote_pair_action(
    board: &Board,
    pair: DigitSet,
    d1: Digit,
    d2: Digit,
    c1: Cell,
    c2: Cell,
    chute: ChuteInfo,
    effects: &mut Effects,
    single: bool,
) -> bool {
    let present_d1 = digit_present(board, chute.yellow_cells, d1);
    let present_d2 = digit_present(board, chute.yellow_cells, d2);

    if present_d1 && present_d2 {
        return false;
    }

    let common_peers = c1.peers() & c2.peers();
    let mut action = Action::new(Strategy::ChuteRemotePair);
    let mut has_change = false;

    if present_d1 ^ present_d2 {
        let erase_digit = if present_d1 { d1 } else { d2 };
        let erase = common_peers & board.candidate_cells(erase_digit);
        if !erase.is_empty() {
            action.erase_cells(erase, erase_digit);
            has_change = true;
        }
    } else {
        for digit in [d1, d2] {
            let erase = common_peers & board.candidate_cells(digit);
            if !erase.is_empty() {
                action.erase_cells(erase, digit);
                has_change = true;
            }
        }

        let bonus_cells = bonus_cells(c1, c2, chute.kind) - c1 - c2;
        for digit in [d1, d2] {
            let erase = bonus_cells & board.candidate_cells(digit);
            if !erase.is_empty() {
                action.erase_cells(erase, digit);
                has_change = true;
            }
        }
    }

    if !has_change {
        return false;
    }

    action.clue_cells_for_digits(Verdict::Primary, CellSet::of(&[c1, c2]), pair);
    action.clue_cells_for_digits(Verdict::Secondary, chute.yellow_cells, pair);

    effects.add_action(action) && single
}

fn digit_present(board: &Board, cells: CellSet, digit: Digit) -> bool {
    cells.iter().any(|cell| match board.value(cell).digit() {
        Some(value) => value == digit,
        None => board.is_candidate(cell, digit),
    })
}

fn bonus_cells(c1: Cell, c2: Cell, kind: ChuteKind) -> CellSet {
    let mut cells = CellSet::empty();
    match kind {
        ChuteKind::Band => {
            let row1 = c1.row();
            let row2 = c2.row();
            for block in [c1.block(), c2.block()] {
                cells |= block.intersect(row1);
                cells |= block.intersect(row2);
            }
        }
        ChuteKind::Stack => {
            let col1 = c1.column();
            let col2 = c2.column();
            for block in [c1.block(), c2.block()] {
                cells |= block.intersect(col1);
                cells |= block.intersect(col2);
            }
        }
    }
    cells
}

#[derive(Clone, Copy)]
struct ChuteInfo {
    kind: ChuteKind,
    yellow_cells: CellSet,
}

#[derive(Clone, Copy)]
enum ChuteKind {
    Band,
    Stack,
}

fn chute_info(c1: Cell, c2: Cell) -> Option<ChuteInfo> {
    let block1 = c1.block_coord().u8();
    let block2 = c2.block_coord().u8();

    if block1 / 3 == block2 / 3 {
        let band = block1 / 3;
        let unused_block = unused_block_in_band(band, block1, block2);
        let row3 = third_coord_in_band(band, c1.row_coord().u8(), c2.row_coord().u8());
        let yellow_row = House::row(Coord::new(row3));
        let yellow_cells = yellow_row.intersect(House::block(Coord::new(unused_block)));
        return Some(ChuteInfo {
            kind: ChuteKind::Band,
            yellow_cells,
        });
    }

    if block1 % 3 == block2 % 3 {
        let stack = block1 % 3;
        let unused_block = unused_block_in_stack(stack, block1, block2);
        let col3 = third_coord_in_stack(stack, c1.column_coord().u8(), c2.column_coord().u8());
        let yellow_column = House::column(Coord::new(col3));
        let yellow_cells = yellow_column.intersect(House::block(Coord::new(unused_block)));
        return Some(ChuteInfo {
            kind: ChuteKind::Stack,
            yellow_cells,
        });
    }

    None
}

fn unused_block_in_band(band: u8, block1: u8, block2: u8) -> u8 {
    let start = band * 3;
    for offset in 0..3 {
        let block = start + offset;
        if block != block1 && block != block2 {
            return block;
        }
    }
    block1
}

fn unused_block_in_stack(stack: u8, block1: u8, block2: u8) -> u8 {
    for offset in 0..3 {
        let block = stack + offset * 3;
        if block != block1 && block != block2 {
            return block;
        }
    }
    block1
}

fn third_coord_in_band(band: u8, first: u8, second: u8) -> u8 {
    let start = band * 3;
    for offset in 0..3 {
        let value = start + offset;
        if value != first && value != second {
            return value;
        }
    }
    first
}

fn third_coord_in_stack(stack: u8, first: u8, second: u8) -> u8 {
    let start = stack * 3;
    for offset in 0..3 {
        let value = start + offset;
        if value != first && value != second {
            return value;
        }
    }
    first
}

#[cfg(test)]
mod tests {
    use crate::io::{Parse, Parser};
    use crate::*;

    use super::*;

    #[test]
    fn example_1() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B2j0b5v0962050f2i0c0i5u0e560c566201022i060c0a644c62050i03095u4a054c2d062b0e5u0f03440a0i2c040l040l0g060i0c08052c032c0e4a560r091f08050i1m010c2k2k360f0a0d020i070e0c0h",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_chute_remote_pairs(&board, false).expect("not found");
        assert_erase!(got, A1, 4);
        assert_erase!(got, C7, 4);
    }

    #[test]
    fn example_2() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B2b0b5v0962050f2i0c0i5u0e560c566201020d060c0a5w445u050i03095u4a054c2d062b0e5u0f03440a0i2c040l040l0g060i0c08052c032c0e4a560r091f08050i1m010c2k2k360f0a0d020i070e0c0h",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_chute_remote_pairs(&board, false).expect("not found");
        assert_erase!(got, C5, 2);
        assert_erase!(got, D6, 2);
    }

    #[test]
    fn example_3() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B0c057u0f0h0b7u010g0a7u0f050703bg0sb80g02080i0d01030f0e06d69q020e9eb72eb7b6019g0d0c9ec405c40e9e9k0a06087o2e040b0f05080a0d07090c4a5u2b0c09060e0s0l7u037n0g0b0e1n081f",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_chute_remote_pairs(&board, false).expect("not found");
        assert_erase!(got, D2, 9);
        assert_erase!(got, D3, 9);
    }

    #[test]
    fn double_eliminations() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B310z31030i060h102i0306302s4i6i01092i2i0i0801102i100c0f2302273m1z092i080n090h1r381j2b2i0l050z0713044p4j7o067r2r0c2r0i0d02060z0h0h04091u1v0z0c07021g0z1g080g03820d7n",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_chute_remote_pairs(&board, false).expect("not found");
        assert_erase!(got, B3, 4);
        assert_erase!(got, B3, 7);
    }

    #[test]
    fn bonus_eliminations() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B8i058i0801070304020207030405090108060108040206039e9e05050308aa0401acac9eaa8i02030805aa01040401aaaa9g1g050308du8i0105035004ac9e030405aa9g50dwac01du02aa019e04du0503",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_chute_remote_pairs(&board, false).expect("not found");
        assert_erase!(got, J7, 7);
        assert_erase!(got, J7, 9);
        assert_erase!(got, G8, 7);
        assert_erase!(got, G8, 9);
    }
}
