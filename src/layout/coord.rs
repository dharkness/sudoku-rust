#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Coord(u8);

impl Coord {
    pub const fn new(coord: u8) -> Self {
        debug_assert!(coord <= 8);
        Self(coord)
    }

    pub const fn u8(&self) -> u8 {
        self.0
    }

    pub const fn u32(&self) -> u32 {
        self.0 as u32
    }

    pub const fn usize(&self) -> usize {
        self.0 as usize
    }
}
