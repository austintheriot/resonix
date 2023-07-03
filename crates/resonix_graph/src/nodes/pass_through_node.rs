use std::any::Any;

use petgraph::prelude::EdgeIndex;
use uuid::Uuid;

use crate::{AddConnectionError, AudioContext, Connection, Node, NodeType};

/// Takes one signal and passed it through, unaltered
/// to all connected outputs.
///
/// Input 0 - Input signal
///
/// Output 0 - Unaltered Input signal
#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct PassThroughNode {
    uid: u32,
    incoming_connection_indexes: Vec<EdgeIndex>,
    outgoing_connection_indexes: Vec<EdgeIndex>,
}

impl AudioContext {
    pub fn new_pass_through_node(&mut self) -> PassThroughNode {
        PassThroughNode {
            uid: self.new_node_uid(),
            ..Default::default()
        }
    }
}

impl PassThroughNode {
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

impl Node for PassThroughNode {
    #[inline]
    fn process(
        &mut self,
        inputs: &mut dyn Iterator<Item = &Connection>,
        outputs: &mut dyn Iterator<Item = &mut Connection>,
    ) {
        let input_data = inputs.next().map(|c| c.data()).unwrap_or(0.0);

        // copy first input to all output connections
        for output in outputs.into_iter() {
            output.set_data(input_data);
        }
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

    fn uid(&self) -> u32 {
        self.uid
    }

    fn set_uid(&mut self, uid: u32) {
        self.uid = uid;
    }

    fn name(&self) -> String {
        String::from("PassThroughNode")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn incoming_connection_indexes(&self) -> &[EdgeIndex] {
        &self.incoming_connection_indexes
    }

    fn outgoing_connection_indexes(&self) -> &[EdgeIndex] {
        &self.outgoing_connection_indexes
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
        edge_index: EdgeIndex,
    ) -> Result<(), AddConnectionError> {
        self.outgoing_connection_indexes.push(edge_index);

        Ok(())
    }
}

#[cfg(test)]
mod test_pass_through_node {

    use crate::{AudioContext, Connection, Node, PassThroughNode};

    #[test]
    fn should_pass_audio_data_through_output_connections() {
        let mut audio_context = AudioContext::new();
        let mut pass_through_node = audio_context.new_pass_through_node();

        let input_connection = Connection::from_test_data(0, 0.1234, 0, 0);

        let mut output_connection = Connection::default();

        // before processing, output connection holds 0.0
        {
            assert_eq!(output_connection.data(), 0.0);
        }

        {
            let inputs = [&input_connection];
            let outputs = [&mut output_connection];
            pass_through_node.process(&mut inputs.into_iter(), &mut outputs.into_iter())
        }

        // before processing, output connection holds input data
        {
            assert_eq!(output_connection.data(), 0.1234);
        }
    }
}
