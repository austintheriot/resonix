use std::{
    any::Any,
    hash::{Hash, Hasher},
};

use petgraph::prelude::EdgeIndex;
use uuid::Uuid;

use crate::{AddConnectionError, Connection, Node, NodeType};

/// Takes no input signals and outputs a single,
/// constant signal value to all output connections.
///
/// Output 0 - Constant signal value
#[derive(Debug, Clone)]
pub struct ConstantNode {
    uuid: Uuid,
    signal_value: f32,
    outgoing_connection_indexes: Vec<EdgeIndex>,
}

impl ConstantNode {
    pub fn new() -> Self {
        Self::new_with_signal_value(0.0)
    }

    pub fn new_with_signal_value(signal_value: f32) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            signal_value,
            outgoing_connection_indexes: Vec::new(),
        }
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

    fn uuid(&self) -> &Uuid {
        &self.uuid
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

impl Default for ConstantNode {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            signal_value: 0.0,
            outgoing_connection_indexes: Vec::new(),
        }
    }
}

impl PartialEq for ConstantNode {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for ConstantNode {}

impl PartialOrd for ConstantNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.uuid.partial_cmp(&other.uuid)
    }
}

impl Ord for ConstantNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uuid.cmp(&other.uuid)
    }
}

impl Hash for ConstantNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}

#[cfg(test)]
mod test_constant_node {

    use crate::{Connection, ConstantNode, Node};

    #[test]
    fn should_output_constant_signal_value() {
        let mut constant_node = ConstantNode::new_with_signal_value(1.234);

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
