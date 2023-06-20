use std::cell::{Ref, RefCell, RefMut};

use uuid::Uuid;

use crate::{AudioContext, Connect, Connection, Node, NodeType};

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

        audio_context.add_node(RefCell::new(Box::new(new_pass_through_node.clone())));

        new_pass_through_node
    }
}

impl Node for PassThroughNode {
    fn process(&mut self, inputs: &[Ref<Connection>], outputs: &mut [RefMut<Connection>]) {
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
}

impl Connect for PassThroughNode {
    fn connect<N: Node + Connect + Clone>(&self, other_node: &N) -> &Self {
        self.audio_context
            .connect_nodes(self.clone(), other_node.clone());
        self
    }

    fn connect_nodes_with_indexes<N: Node + Connect + Clone>(
        &self,
        from_index: usize,
        other_node: &N,
        to_index: usize,
    ) -> &Self {
        self.audio_context.connect_nodes_with_indexes(
            self.clone(),
            from_index,
            other_node.clone(),
            to_index,
        );
        self
    }
}

#[cfg(test)]
mod test_pass_through_node {
    use std::cell::RefCell;

    use crate::{AudioContext, Connection, ConnectionInner, Node, PassThroughNode};

    #[test]
    fn should_pass_audio_data_through_output_connections() {
        let mut audio_context = AudioContext::new();
        let mut pass_through_node = PassThroughNode::new(&mut audio_context);

        let input_connection = RefCell::new(Connection::from_connection_inner(ConnectionInner {
            from_index: 0,
            to_index: 0,
            data: 0.1234,
            init: true,
        }));

        let output_connection = RefCell::new(Connection::from_connection_inner(ConnectionInner {
            from_index: 0,
            to_index: 0,
            data: 0.0,
            init: false,
        }));

        // before processing, output connection holds 0.0
        {
            let output_connection_ref = output_connection.borrow();
            assert_eq!(output_connection_ref.data(), 0.0);
            assert!(!output_connection_ref.init());
        }

        {
            let incoming_connection_ref = input_connection.borrow();
            let inputs = [incoming_connection_ref];
            let output_connection_ref = output_connection.borrow_mut();
            let mut outputs = [output_connection_ref];
            pass_through_node.process(&inputs, &mut outputs)
        }

        // before processing, output connection holds input data
        {
            let output_connection_ref = output_connection.borrow();
            assert_eq!(output_connection_ref.data(), 0.1234);
            assert!(output_connection_ref.init());
        }
    }
}
