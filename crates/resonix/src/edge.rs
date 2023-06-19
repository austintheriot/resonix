pub type InputIndex = usize;

pub type OutputIndex = usize;

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct Edge {
    data_for_input: Vec<(InputIndex, OutputIndex, Option<f32>)>
}

impl Edge {
    pub fn new() -> Self {
        Self::default()
    }
}