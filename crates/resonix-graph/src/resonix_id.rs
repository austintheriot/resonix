#[derive(Copy, Clone)]
pub struct ResonixId(u32);

impl ResonixId {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

// other convenience implementations possible here

impl From<i32> for ResonixId {
    fn from(value: i32) -> Self {
        ResonixId(value as u32)
    }
}

impl From<u32> for ResonixId {
    fn from(value: u32) -> Self {
        ResonixId(value)
    }
}
