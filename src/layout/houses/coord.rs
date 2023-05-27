#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
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

impl From<i32> for Coord {
    fn from(coord: i32) -> Self {
        assert!(coord >= 0 && coord <= 8);
        Self(coord as u8)
    }
}

impl From<u8> for Coord {
    fn from(coord: u8) -> Self {
        assert!(coord >= 0 && coord <= 8);
        Self(coord as u8)
    }
}

impl From<usize> for Coord {
    fn from(coord: usize) -> Self {
        assert!(coord >= 0 && coord <= 8);
        Self(coord as u8)
    }
}
