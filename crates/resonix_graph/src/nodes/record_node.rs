use std::{any::Any, cell::{Ref, RefMut}};

use petgraph::prelude::EdgeIndex;
use uuid::Uuid;

use crate::{AddConnectionError, Connection, Node, NodeType};

#[derive(Debug, Clone)]
pub struct RecordNode {
    data: Vec<f32>,
    uuid: Uuid,
    incoming_connection_indexes: Vec<EdgeIndex>,
}

impl RecordNode {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn data(&self) -> &Vec<f32> {
        &self.data
    }
}

impl Node for RecordNode {
    #[inline]
    fn process(
        &mut self,
        inputs: &mut dyn Iterator<Item = Ref<Connection>>,
        _: &mut dyn Iterator<Item = RefMut<Connection>>,
    ) {
        let Some(first_input) = inputs.next() else {
            return
        };

        self.data.push(first_input.data());
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
        String::from("RecordNode")
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

impl Default for RecordNode {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            data: Vec::new(),
            incoming_connection_indexes: Vec::new(),
        }
    }
}

impl PartialEq for RecordNode {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for RecordNode {}

impl PartialOrd for RecordNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.uuid.partial_cmp(&other.uuid)
    }
}

impl Ord for RecordNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uuid.cmp(&other.uuid)
    }
}

#[cfg(test)]
mod test_record_node {

    use crate::{Connection, Node, RecordNode};

    #[test]
    fn should_record_incoming_node_data() {
        let mut record_node = RecordNode::new();

        let input_connection = Connection::from_test_data(0.1234, 0, 0);

        {
            let inputs = [&input_connection];
            let outputs = [];
            record_node.process(&mut inputs.into_iter(), &mut outputs.into_iter())
        }

        assert_eq!(record_node.data().len(), 1);
        assert_eq!(*record_node.data().first().unwrap(), 0.1234);
    }
}
