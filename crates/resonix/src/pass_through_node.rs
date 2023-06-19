use std::cell::{Ref, RefMut, RefCell};

use uuid::Uuid;

use crate::{Node, NodeType, Connection, AudioContext, Connect};

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct PassThroughNode {
    uuid: Uuid,
    audio_context: AudioContext,
}

impl PassThroughNode {
    pub fn new(audio_context: &mut AudioContext) -> Self {
        let new_pass_through_node = Self { uuid: Uuid::new_v4(), audio_context: audio_context.clone() };

        audio_context.add_node(RefCell::new(Box::new(new_pass_through_node.clone())));

        new_pass_through_node
    }
}

impl Node for PassThroughNode {
    fn process(&mut self, inputs: &[Ref<Connection>], outputs: &mut [RefMut<Connection>]) {
        outputs[0].data = inputs[0].data;
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

    fn uuid(&self) -> &Uuid {
        &self.uuid
    }
}

impl Connect for PassThroughNode {
    fn connect<N: Node + Connect + Clone>(&self, other_node: &N) -> &Self {
        self.audio_context.connect_nodes(self.clone(), other_node.clone());
        self
    }

    fn connect_nodes_with_indexes<N: Node + Connect + Clone>(&self, from_index: usize, other_node: &N, to_index: usize) -> &Self {
        self.audio_context.connect_nodes_with_indexes(self.clone(), from_index, other_node.clone(), to_index);
        self
    }
}