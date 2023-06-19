use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use uuid::Uuid;

use crate::{AudioContext, Connect, Connection, Node, NodeType};

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct RecordNode {
    data: Rc<RefCell<Vec<f32>>>,
    uuid: Uuid,
    audio_context: AudioContext,
}

impl RecordNode {
    pub fn new(audio_context: &mut AudioContext) -> Self {
        let new_record_node = Self {
            uuid: Uuid::new_v4(),
            audio_context: audio_context.clone(),
            data: Rc::new(RefCell::new(Vec::new())),
        };

        audio_context.add_node(RefCell::new(Box::new(new_record_node.clone())));

        new_record_node
    }

    pub fn data(&self) -> Ref<Vec<f32>> {
        self.data.borrow()
    }
}

impl Node for RecordNode {
    fn process(&mut self, inputs: &[Ref<Connection>], _outputs: &mut [RefMut<Connection>]) {
        let Some(first_input) = inputs.first() else {
            return
        };

        self.data.borrow_mut().push(first_input.data);
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
}

impl Connect for RecordNode {
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
mod test_record_node {
    use std::cell::RefCell;

    use crate::{AudioContext, Connection, MultiplyNode, Node, RecordNode};

    #[test]
    fn should_record_incoming_node_data() {
        let mut audio_context = AudioContext::new();
        let mut record_node = RecordNode::new(&mut audio_context);

        let input_connection = RefCell::new(Connection {
            from_index: 0,
            to_index: 0,
            data: 0.1234,
            init: true,
        });

        {
            let incoming_connection_ref = input_connection.borrow();
            let inputs = [incoming_connection_ref];
            let mut outputs = [];
            record_node.process(&inputs, &mut outputs)
        }

        let record_data = record_node.data.borrow();
        assert_eq!(record_data.len(), 1);
        assert_eq!(*record_data.first().unwrap(), 0.1234);
    }
}
