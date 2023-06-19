use std::cell::{Ref, RefCell, RefMut};

use uuid::Uuid;

use crate::{AudioContext, Connect, Connection, Node, NodeType};

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

        audio_context.add_node(RefCell::new(Box::new(new_multiply_node.clone())));

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

    fn process(&mut self, inputs: &[Ref<Connection>], outputs: &mut [RefMut<Connection>]) {
        outputs[0].data = inputs[0].data * inputs[1].data;
    }

    fn uuid(&self) -> &Uuid {
        &self.uuid
    }
}

impl Connect for MultiplyNode {
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
mod test_multiply_node {
    use std::cell::RefCell;

    use crate::{AudioContext, Connection, MultiplyNode, Node};

    #[test]
    fn should_multiply_1st_and_2nd_inputs() {
        let mut audio_context = AudioContext::new();
        let mut multiply_node = MultiplyNode::new(&mut audio_context);

        let left_input_connection = RefCell::new(Connection {
            from_index: 0,
            to_index: 0,
            data: 0.5,
            init: true,
        });

        let right_input_connection = RefCell::new(Connection {
            from_index: 0,
            to_index: 1,
            data: 0.2,
            init: true,
        });

        let output_connection = RefCell::new(Connection {
            from_index: 0,
            to_index: 0,
            data: 0.0,
            init: false,
        });

        {
            let left_input_connection_ref = left_input_connection.borrow();
            let right_input_connection_ref = right_input_connection.borrow();
            let output_connection_ref_mut = output_connection.borrow_mut();
            let inputs = [left_input_connection_ref, right_input_connection_ref];
            let mut outputs = [output_connection_ref_mut];
            multiply_node.process(&inputs, &mut outputs)
        }

        assert_eq!(output_connection.borrow().data, 0.1);
    }
}
