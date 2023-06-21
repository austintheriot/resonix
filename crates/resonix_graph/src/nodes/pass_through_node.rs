use std::any::Any;

use uuid::Uuid;

use crate::{AudioContext, Connect, ConnectError, Connection, Node, NodeType};

/// Takes one signal and passed it through, unaltered
/// to all connected outputs.
///
/// Input 0 - Input signal
///
/// Output 0 - Unaltered Input signal
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct PassThroughNode {
    uuid: Uuid,
    audio_context: AudioContext,
}

impl PassThroughNode {
    pub fn new(audio_context: &mut AudioContext) -> Self {
        let new_pass_through_node = Self {
            uuid: Uuid::new_v4(),
            audio_context: audio_context.clone(),
        };

        audio_context.add_node(new_pass_through_node.clone());

        new_pass_through_node
    }
}

impl Node for PassThroughNode {
    fn process(&mut self, inputs: &[Connection], outputs: &mut [Connection]) {
        let input_data = inputs[0].data();

        // copy first input to all output connections
        for output in outputs.iter_mut() {
            output.set_data(input_data);
            output.set_init(true);
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

    fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    fn name(&self) -> String {
        String::from("PassThroughNode")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Connect for PassThroughNode {
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
mod test_pass_through_node {

    use crate::{AudioContext, Connection, ConnectionInner, Node, PassThroughNode};

    #[test]
    fn should_pass_audio_data_through_output_connections() {
        let mut audio_context = AudioContext::new();
        let mut pass_through_node = PassThroughNode::new(&mut audio_context);

        let input_connection = Connection::from_connection_inner(ConnectionInner {
            from_index: 0,
            to_index: 0,
            data: 0.1234,
            init: true,
        });

        let output_connection = Connection::from_connection_inner(ConnectionInner {
            from_index: 0,
            to_index: 0,
            data: 0.0,
            init: false,
        });

        // before processing, output connection holds 0.0
        {
            assert_eq!(output_connection.data(), 0.0);
            assert!(!output_connection.init());
        }

        {
            let inputs = [input_connection];
            let mut outputs = [output_connection.clone()];
            pass_through_node.process(&inputs, &mut outputs)
        }

        // before processing, output connection holds input data
        {
            assert_eq!(output_connection.data(), 0.1234);
            assert!(output_connection.init());
        }
    }
}
