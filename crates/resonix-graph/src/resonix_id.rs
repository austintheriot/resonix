#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResonixId(usize);

impl ResonixId {
    pub const fn new(id: usize) -> Self {
        Self(id)
    }
}

// other convenience implementations possible here

impl From<i32> for ResonixId {
    fn from(value: i32) -> Self {
        ResonixId(value as usize)
    }
}

impl From<usize> for ResonixId {
    fn from(value: usize) -> Self {
        ResonixId(value)
    }
}
