use std::any::Any;

use petgraph::prelude::EdgeIndex;
use uuid::Uuid;

use crate::{AddConnectionError, Connection, Node, NodeType, AudioContext};

/// Takes two signals and multiplies them together,
/// outputting the signal to all connected outputs
///
/// Input 0 - Signal 1
/// Input 1 - Signal 2
///
/// Output 0 - Multiplied signal
#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct MultiplyNode {
    uid: u32,
    incoming_connection_indexes: Vec<EdgeIndex>,
    outgoing_connection_indexes: Vec<EdgeIndex>,
}

impl AudioContext {
    pub fn new_multiply_node(&mut self) -> MultiplyNode {
        MultiplyNode { uid: self.new_node_uid(), ..Default::default() }
    }
}

impl MultiplyNode {
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

impl Node for MultiplyNode {
    fn node_type(&self) -> crate::NodeType {
        NodeType::Effect
    }

    fn num_inputs(&self) -> usize {
        2
    }

    fn num_outputs(&self) -> usize {
        1
    }

    #[inline]
    fn process(
        &mut self,
        inputs: &mut dyn Iterator<Item = &Connection>,
        outputs: &mut dyn Iterator<Item = &mut Connection>,
    ) {
        let first_input = inputs.next().unwrap();
        let second_input = inputs.next().unwrap();
        let result = first_input.data() * second_input.data();

        // copy to all output connections
        outputs.into_iter().for_each(|output| {
            output.set_data(result);
        })
    }

    fn uid(&self) -> u32 {
        self.uid
    }

    fn set_uid(&mut self, uid: u32) {
        self.uid = uid;
    }

    fn name(&self) -> String {
        String::from("MultiplyNode")
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
mod test_multiply_node {

    use crate::{Connection, Node, AudioContext};

    #[test]
    fn should_multiply_1st_and_2nd_inputs() {
        let mut audio_context = AudioContext::new();
        let mut multiply_node = audio_context.new_multiply_node();

        let left_input_connection = Connection::from_test_data(0, 0.5, 0, 0);
        let right_input_connection = Connection::from_test_data(1, 0.2, 0, 1);
        let mut output_connection = Connection::default();

        // before processing, output data is 0.0
        {
            assert_eq!(output_connection.data(), 0.0);
        }

        // run processing for node
        {
            let inputs = [&left_input_connection, &right_input_connection];
            let outputs = [&mut output_connection];
            multiply_node.process(&mut inputs.into_iter(), &mut outputs.into_iter())
        }

        // before processing, output data is 0.1
        {
            assert_eq!(output_connection.data(), 0.1);
        }
    }
}
