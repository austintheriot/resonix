use std::any::Any;

use petgraph::prelude::EdgeIndex;
use uuid::Uuid;

use crate::{AddConnectionError, Connection, Node, NodeType};

#[derive(Debug, Clone)]
pub struct DACNode {
    data: f32,
    uuid: Uuid,
    incoming_connection_indexes: Vec<EdgeIndex>,
}

impl DACNode {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn data(&self) -> f32 {
        self.data
    }
}

impl Node for DACNode {
    fn process(
        &mut self,
        inputs: &mut dyn Iterator<Item = &Connection>,
        _outputs: &mut dyn Iterator<Item = &mut Connection>,
    ) {
        let Some(first_input) = inputs.next() else {
            return
        };

        let sample = first_input.data();

        self.data = sample;
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

    fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    fn name(&self) -> String {
        String::from("DACNode")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn incoming_connection_indexes(&self) -> &[petgraph::prelude::EdgeIndex] {
        &self.incoming_connection_indexes
    }

    fn outgoing_connection_indexes(&self) -> &[EdgeIndex] {
        &[]
    }

    fn add_incoming_connection_index(
        &mut self,
        edge_index: EdgeIndex,
    ) -> Result<(), AddConnectionError> {
        self.incoming_connection_indexes.push(edge_index);

        Ok(())
    }

    fn add_outgoing_connection_index(
        &mut self,
        _edge_index: EdgeIndex,
    ) -> Result<(), AddConnectionError> {
        Err(AddConnectionError::CantAcceptOutputConnections { name: self.name() })
    }
}

impl Default for DACNode {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            data: 0.0,
            incoming_connection_indexes: Vec::new(),
        }
    }
}

impl PartialEq for DACNode {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for DACNode {}

impl PartialOrd for DACNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.uuid.partial_cmp(&other.uuid)
    }
}

impl Ord for DACNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uuid.cmp(&other.uuid)
    }
}

#[cfg(test)]
mod test_dac_node {

    use crate::{Connection, DACNode, Node};

    #[test]
    fn should_record_one_sample_of_incoming_data() {
        let mut dac_node = DACNode::new();

        let input_connection = Connection::from_test_data(0.1234, 0, 0);

        assert_eq!(dac_node.data(), 0.0);

        {
            let inputs = [&input_connection];
            let outputs = [];
            dac_node.process(&mut inputs.into_iter(), &mut outputs.into_iter())
        }

        assert_eq!(dac_node.data(), 0.1234);
    }
}
