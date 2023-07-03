use std::any::Any;

use petgraph::prelude::EdgeIndex;
use uuid::Uuid;

use crate::{AddConnectionError, Connection, Node, NodeType, AudioContext};

#[derive(Debug, Clone, Default)]
pub struct DACNode {
    data: f32,
    uid: u32,
    incoming_connection_indexes: Vec<EdgeIndex>,
}

impl AudioContext {
    pub fn new_dac_node(&mut self) -> DACNode {
        DACNode { uid: self.new_node_uid(), ..Default::default() }
    }
}

impl DACNode {
    pub fn data(&self) -> f32 {
        self.data
    }

    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub(crate) fn new_with_uid(uid: u32) -> Self {
        Self {
            uid,
            ..Default::default()
        }
    }
}

impl Node for DACNode {
    #[inline]
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

    fn uid(&self) -> u32 {
        self.uid
    }

    fn set_uid(&mut self, uid: u32) {
        self.uid = uid;
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

impl PartialEq for DACNode {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl Eq for DACNode {}

impl PartialOrd for DACNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.uid.partial_cmp(&other.uid)
    }
}

impl Ord for DACNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uid.cmp(&other.uid)
    }
}

#[cfg(test)]
mod test_dac_node {

    use crate::{Connection, DACNode, Node, AudioContext};

    #[test]
    fn should_record_one_sample_of_incoming_data() {
        let mut audio_context = AudioContext::new();
        let mut dac_node = audio_context.new_dac_node();

        let input_connection = Connection::from_test_data(0, 0.1234, 0, 0);

        assert_eq!(dac_node.data(), 0.0);

        {
            let inputs = [&input_connection];
            let outputs = [];
            dac_node.process(&mut inputs.into_iter(), &mut outputs.into_iter())
        }

        assert_eq!(dac_node.data(), 0.1234);
    }
}
