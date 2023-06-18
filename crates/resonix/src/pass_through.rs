use crate::{Node, NodeType};

#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct PassThrough;

impl Node for PassThrough {
    fn process(&mut self, inputs: &[f32], outputs: &mut [f32]) {
        outputs
            .iter_mut()
            .enumerate()
            .for_each(|(i, o)| *o = inputs[i]);
    }

    fn node_type(&self) -> NodeType {
        NodeType::Effect
    }

    fn num_inputs(&self) -> usize {
        1
    }

    fn num_outputs(&self) -> usize {
        1
    }
}
