use std::{
    any::Any,
    hash::{Hash, Hasher},
};

use petgraph::prelude::EdgeIndex;

use crate::{AddConnectionError, Connection, Node, NodeType, AudioContext};

/// Takes no input signals and outputs a single,
/// constant signal value to all output connections.
///
/// Output 0 - Constant signal value
#[derive(Debug, Clone, Default)]
pub struct ConstantNode {
    uid: u32,
    signal_value: f32,
    outgoing_connection_indexes: Vec<EdgeIndex>,
}

impl AudioContext {
    pub fn new_constant_node(&mut self, signal_value: f32) -> ConstantNode {
        let mut constant_node = ConstantNode::new(signal_value);
        constant_node.uid = self.new_node_uid();
        constant_node
    }
}


impl ConstantNode {
    pub fn new(signal_value: f32) -> Self {
        Self { signal_value, ..Default::default() }
    }

    #[cfg(test)]
    pub fn new_with_uid(uid: u32, signal_value: f32, ) -> Self {
        Self { signal_value, uid, ..Default::default() }
    }

    pub fn signal_value(&self) -> f32 {
        self.signal_value
    }

    pub fn set_signal_value(&mut self, signal_value: f32) -> &mut Self {
        self.signal_value = signal_value;
        self
    }
}

impl Node for ConstantNode {
    fn node_type(&self) -> crate::NodeType {
        NodeType::Input
    }

    fn num_inputs(&self) -> usize {
        0
    }

    fn num_outputs(&self) -> usize {
        1
    }

    #[inline]
    fn process(
        &mut self,
        _inputs: &mut dyn Iterator<Item = &Connection>,
        outputs: &mut dyn Iterator<Item = &mut Connection>,
    ) {
        // copy to all output connections
        outputs.into_iter().for_each(|output| {
            output.set_data(self.signal_value);
        })
    }

    fn uid(&self) -> u32 {
        self.uid
    }

    fn set_uid(&mut self, uid: u32) {
        self.uid = uid;
    }

    fn name(&self) -> String {
        String::from("ConstantNode")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn incoming_connection_indexes(&self) -> &[EdgeIndex] {
        &[]
    }

    fn outgoing_connection_indexes(&self) -> &[EdgeIndex] {
        &self.outgoing_connection_indexes
    }

    fn add_incoming_connection_index(
        &mut self,
        _edge_index: EdgeIndex,
    ) -> Result<(), AddConnectionError> {
        Err(AddConnectionError::CantAcceptInputConnections { name: self.name() })
    }

    fn add_outgoing_connection_index(
        &mut self,
        edge_index: EdgeIndex,
    ) -> Result<(), AddConnectionError> {
        self.outgoing_connection_indexes.push(edge_index);

        Ok(())
    }
}


impl PartialEq for ConstantNode {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl Eq for ConstantNode {}

impl PartialOrd for ConstantNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.uid.partial_cmp(&other.uid)
    }
}

impl Ord for ConstantNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uid.cmp(&other.uid)
    }
}

impl Hash for ConstantNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uid.hash(state);
    }
}

#[cfg(test)]
mod test_constant_node {

    use crate::{Connection, Node, AudioContext};

    #[test]
    fn should_output_constant_signal_value() {
        let mut audio_context = AudioContext::new();
        let mut constant_node = audio_context.new_constant_node(1.234);

        let mut output_connection = Connection::default();

        // before processing, output data is 0.0
        {
            assert_eq!(output_connection.data(), 0.0);
        }

        // run processing for node
        {
            let inputs = [];
            let outputs = [&mut output_connection];
            constant_node.process(&mut inputs.into_iter(), &mut outputs.into_iter())
        }

        // after processing, output data is 1.234
        {
            assert_eq!(output_connection.data(), 1.234);
        }
    }
}
