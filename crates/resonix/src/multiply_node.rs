use uuid::Uuid;

use crate::{AudioContext, Connect, ConnectError, Connection, Node, NodeType};

/// Takes two signals and multiplies them together,
/// outputting the signal to all connected outputs
///
/// Input 0 - Signal 1
/// Input 1 - Signal 2
///
/// Output 0 - Multiplied signal
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct MultiplyNode {
    uuid: Uuid,
    audio_context: AudioContext,
}

impl MultiplyNode {
    pub fn new(audio_context: &mut AudioContext) -> Self {
        let new_multiply_node = Self {
            uuid: Uuid::new_v4(),
            audio_context: audio_context.clone(),
        };

        audio_context.add_node(new_multiply_node.clone());

        new_multiply_node
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

    fn process(&mut self, inputs: &[Connection], outputs: &mut [Connection]) {
        let result = inputs[0].data() * inputs[1].data();

        // copy to all output connections
        outputs.iter_mut().for_each(|output| {
            output.set_data(result);
            output.set_init(true);
        })
    }

    fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    fn name(&self) -> String {
        String::from("MultiplyNode")
    }
}

impl Connect for MultiplyNode {
    fn connect_nodes_with_indexes<N: Node + Connect + Clone>(
        &self,
        from_index: usize,
        other_node: &N,
        to_index: usize,
    ) -> Result<&Self, ConnectError> {
        self.check_index_out_of_bounds(from_index, other_node, to_index)?;

        self.audio_context.connect_nodes_with_indexes(
            self.clone(),
            from_index,
            other_node.clone(),
            to_index,
        );

        Ok(self)
    }
}

#[cfg(test)]
mod test_multiply_node {

    use crate::{AudioContext, Connection, ConnectionInner, MultiplyNode, Node};

    #[test]
    fn should_multiply_1st_and_2nd_inputs() {
        let mut audio_context = AudioContext::new();
        let mut multiply_node = MultiplyNode::new(&mut audio_context);

        let left_input_connection = Connection::from_connection_inner(ConnectionInner {
            from_index: 0,
            to_index: 0,
            data: 0.5,
            init: true,
        });

        let right_input_connection = Connection::from_connection_inner(ConnectionInner {
            from_index: 0,
            to_index: 1,
            data: 0.2,
            init: true,
        });

        let output_connection = Connection::from_connection_inner(ConnectionInner {
            from_index: 0,
            to_index: 0,
            data: 0.0,
            init: false,
        });

        // before processing, output data is 0.0
        {
            assert_eq!(output_connection.data(), 0.0);
            assert!(!output_connection.init());
        }

        // run processing for node
        {
            let inputs = [left_input_connection, right_input_connection];
            let mut outputs = [output_connection.clone()];
            multiply_node.process(&inputs, &mut outputs)
        }

        // before processing, output data is 0.1
        {
            assert_eq!(output_connection.data(), 0.1);
            assert!(output_connection.init());
        }
    }
}
