use crate::{Node, NodeType};

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct Record {
    data: Vec<f32>,
}

impl Record {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Node for Record {
    fn process(&mut self, inputs: &[f32], _outputs: &mut [f32]) {
        if let Some(sample_from_first_input) = inputs.first() {
            self.data.push(*sample_from_first_input);
        }
    }

    fn node_type(&self) -> NodeType {
        NodeType::Output
    }

    fn num_inputs(&self) -> usize {
        1
    }

    fn num_outputs(&self) -> usize {
        0
    }
}