use std::ops::{Deref, DerefMut};
use crate::layout::cells::label::index_from_label;

use super::Cell;

/// Specifies a single cell by its position in a bit field.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Bit(u128);

impl Bit {
    pub const MAX: u128 = 1 << Cell::COUNT - 1;
    pub const ALL: u128 = (1 << Cell::COUNT) - 1;

    pub const fn new(bit: u128) -> Self {
        debug_assert!(bit <= Bit::MAX && bit.count_ones() == 1);
        Self(bit)
    }

    pub const fn bit(&self) -> u128 {
        self.0
    }

    pub const fn index(&self) -> u8 {
        self.0.trailing_zeros() as u8
    }

    pub const fn cell(&self) -> Cell {
        Cell::new(self.index())
    }
}

impl Deref for Bit {
    type Target = u128;

    fn deref(&self) -> &u128 {
        &self.0
    }
}

impl DerefMut for Bit {
    fn deref_mut(&mut self) -> &mut u128 {
        &mut self.0
    }
}

impl From<&str> for Bit {
    fn from(label: &str) -> Self {
        Self(1 << index_from_label(label))
    }
}

impl From<Cell> for Bit {
    fn from(cell: Cell) -> Self {
        cell.bit()
    }
}
