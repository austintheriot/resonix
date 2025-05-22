#[derive(Debug, PartialEq, PartialOrd, Clone, Default)]
pub enum ResonixData {
    #[default]
    None,
    F32(f32),
    I32(i32),
    Error,
}

impl From<f32> for ResonixData {
    fn from(value: f32) -> Self {
        Self::F32(value)
    }
}

impl From<i32> for ResonixData {
    fn from(value: i32) -> Self {
        Self::I32(value)
    }
}
