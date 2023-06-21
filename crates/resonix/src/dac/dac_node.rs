use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use uuid::Uuid;

use crate::{AudioContext, Connect, ConnectError, Connection, Node, NodeType};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct DACNode {
    data: Rc<RefCell<f32>>,
    uuid: Uuid,
    audio_context: AudioContext,
}

impl DACNode {
    pub fn new(audio_context: &mut AudioContext) -> Self {
        let new_dac_node = Self {
            uuid: Uuid::new_v4(),
            audio_context: audio_context.clone(),
            data: Rc::new(RefCell::new(0.0)),
        };

        audio_context.add_node(new_dac_node.clone());

        new_dac_node
    }

    pub fn data(&self) -> f32 {
        *self.data.borrow()
    }
}

impl Node for DACNode {
    fn process(&mut self, inputs: &[Ref<Connection>], _outputs: &mut [RefMut<Connection>]) {
        let Some(first_input) = inputs.first() else {
            return
        };

        *self.data.borrow_mut() = first_input.data();
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
}

impl Connect for DACNode {
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
mod test_dac_node {
    use std::cell::RefCell;

    use crate::{AudioContext, Connection, ConnectionInner, DACNode, Node};

    #[test]
    fn should_record_one_sample_of_incoming_data() {
        let mut audio_context = AudioContext::new();
        let mut dac_node = DACNode::new(&mut audio_context);

        let input_connection = RefCell::new(Connection::from_connection_inner(ConnectionInner {
            from_index: 0,
            to_index: 0,
            data: 0.1234,
            init: true,
        }));


        assert_eq!(dac_node.data(), 0.0);

        {
            let incoming_connection_ref = input_connection.borrow();
            let inputs = [incoming_connection_ref];
            let mut outputs = [];
            dac_node.process(&inputs, &mut outputs)
        }

        assert_eq!(dac_node.data(), 0.1234);
    }
}
