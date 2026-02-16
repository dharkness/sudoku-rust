use std::iter::FusedIterator;
use std::ops::{Add, AddAssign, Index, Sub, SubAssign};

use crate::layout::Cell;

/// A compact set of unordered cell pairs.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PeerSet {
    bits: [u128; 26],
}

impl PeerSet {
    pub const fn empty() -> Self {
        Self { bits: [0; 26] }
    }

    pub fn is_empty(&self) -> bool {
        self.bits.iter().all(|word| *word == 0)
    }

    pub fn len(&self) -> usize {
        self.bits
            .iter()
            .map(|word| word.count_ones() as usize)
            .sum()
    }

    pub fn has(&self, a: Cell, b: Cell) -> bool {
        debug_assert!(a.sees(b), "PeerSet only stores peer pairs");
        let (word, bit) = pair_index(a, b);
        (self.bits[word] & (1u128 << bit)) != 0
    }

    pub fn contains(&self, a: Cell, b: Cell) -> bool {
        self.has(a, b)
    }

    pub fn add(&mut self, a: Cell, b: Cell) {
        debug_assert!(a.sees(b), "PeerSet only stores peer pairs");
        let (word, bit) = pair_index(a, b);
        self.bits[word] |= 1u128 << bit;
    }

    pub fn remove(&mut self, a: Cell, b: Cell) {
        debug_assert!(a.sees(b), "PeerSet only stores peer pairs");
        let (word, bit) = pair_index(a, b);
        self.bits[word] &= !(1u128 << bit);
    }

    pub fn with(&self, a: Cell, b: Cell) -> Self {
        let mut next = *self;
        next += (a, b);
        next
    }

    pub fn without(&self, a: Cell, b: Cell) -> Self {
        let mut next = *self;
        next.remove(a, b);
        next
    }

    pub fn iter(&self) -> PeerSetIter<'_> {
        PeerSetIter::new(self)
    }
}

impl Add<(Cell, Cell)> for PeerSet {
    type Output = Self;

    fn add(self, rhs: (Cell, Cell)) -> Self {
        self.with(rhs.0, rhs.1)
    }
}

impl AddAssign<(Cell, Cell)> for PeerSet {
    fn add_assign(&mut self, rhs: (Cell, Cell)) {
        PeerSet::add(self, rhs.0, rhs.1);
    }
}

impl Sub<(Cell, Cell)> for PeerSet {
    type Output = Self;

    fn sub(self, rhs: (Cell, Cell)) -> Self {
        self.without(rhs.0, rhs.1)
    }
}

impl SubAssign<(Cell, Cell)> for PeerSet {
    fn sub_assign(&mut self, rhs: (Cell, Cell)) {
        self.remove(rhs.0, rhs.1);
    }
}

impl Index<(Cell, Cell)> for PeerSet {
    type Output = bool;

    fn index(&self, pair: (Cell, Cell)) -> &bool {
        if self.has(pair.0, pair.1) {
            &true
        } else {
            &false
        }
    }
}

pub struct PeerSetIter<'a> {
    set: &'a PeerSet,
    word_index: usize,
    word: u128,
}

impl<'a> PeerSetIter<'a> {
    fn new(set: &'a PeerSet) -> Self {
        let word = set.bits.first().copied().unwrap_or(0);
        Self {
            set,
            word_index: 0,
            word,
        }
    }
}

impl<'a> Iterator for PeerSetIter<'a> {
    type Item = (Cell, Cell);

    fn next(&mut self) -> Option<Self::Item> {
        while self.word_index < self.set.bits.len() {
            if self.word == 0 {
                self.word_index += 1;
                self.word = self.set.bits.get(self.word_index).copied().unwrap_or(0);
                continue;
            }

            let tz = self.word.trailing_zeros() as usize;
            self.word &= self.word - 1;
            let index = self.word_index * 128 + tz;
            return Some(unpair_index(index));
        }
        None
    }
}

impl<'a> FusedIterator for PeerSetIter<'a> {}

impl<'a> IntoIterator for &'a PeerSet {
    type Item = (Cell, Cell);
    type IntoIter = PeerSetIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

const PAIR_ROW_BASE: [usize; 81] = {
    let mut base = [0usize; 81];
    let mut row = 0usize;
    let mut sum = 0usize;
    while row < 81 {
        base[row] = sum;
        sum += 80 - row;
        row += 1;
    }
    base
};

fn pair_index(a: Cell, b: Cell) -> (usize, u8) {
    let mut i = a.usize();
    let mut j = b.usize();
    if i == j {
        debug_assert!(i != j, "PeerSet does not allow identical cells");
        return (0, 0);
    }
    if i > j {
        std::mem::swap(&mut i, &mut j);
    }

    let index = PAIR_ROW_BASE[i] + (j - i - 1);
    let word = index / 128;
    let bit = (index % 128) as u8;
    (word, bit)
}

fn unpair_index(index: usize) -> (Cell, Cell) {
    let mut row = 0usize;
    let mut remaining = index;
    while row < 80 {
        let count = 80 - row;
        if remaining < count {
            let col = row + 1 + remaining;
            return (Cell::new(row as u8), Cell::new(col as u8));
        }
        remaining -= count;
        row += 1;
    }

    debug_assert!(false, "invalid pair index {}", index);
    (Cell::new(0), Cell::new(1))
}

#[cfg(test)]
mod tests {
    use crate::layout::Cell;
    use crate::layout::PeerSet;
    use crate::*;

    #[test]
    fn empty_is_empty() {
        let set = PeerSet::empty();
        assert!(set.is_empty());
        assert_eq!(0, set.len());
        assert_eq!(0, set.iter().count());
    }

    #[test]
    fn insert_contains_remove() {
        let mut set = PeerSet::empty();
        let a = cell!(A1);
        let b = cell!(A2);

        set += (a, b);
        assert!(set.has(a, b));
        assert!(set[(b, a)]);
        assert_eq!(1, set.len());

        set -= (a, b);
        assert!(!set.has(a, b));
        assert!(set.is_empty());
    }

    #[test]
    fn insert_is_idempotent() {
        let mut set = PeerSet::empty();
        let a = cell!(B3);
        let b = cell!(B7);

        set += (a, b);
        set += (b, a);
        assert_eq!(1, set.len());
    }

    #[test]
    fn iter_matches_contents() {
        let mut set = PeerSet::empty();
        let pairs = vec![
            (cell!(A1), cell!(A2)),
            (cell!(A1), cell!(B1)),
            (cell!(C3), cell!(A3)),
            (cell!(H4), cell!(J4)),
        ];
        for (a, b) in &pairs {
            set += (*a, *b);
        }

        assert_eq!(pairs.len(), set.iter().count());
        for (a, b) in pairs {
            assert!(set.has(a, b));
        }
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic]
    fn insert_requires_peers() {
        let mut set = PeerSet::empty();
        set += (cell!(A1), cell!(C4));
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic]
    fn contains_requires_peers() {
        let set = PeerSet::empty();
        let _ = set.has(cell!(A1), cell!(C4));
    }
}
