use std::{
    cell::{Ref, RefCell, RefMut},
    hash::{Hash, Hasher},
    rc::Rc,
};

use uuid::Uuid;

use crate::{AudioContext, Connect, Connection, Node, NodeType};

/// Takes no input signals and outputs a single,
/// constant signal value to all output connections.
///
/// Output 0 - Constant signal value
#[derive(Debug, Clone)]
pub struct ConstantNode {
    uuid: Uuid,
    audio_context: AudioContext,
    signal_value: Rc<RefCell<f32>>,
}

impl ConstantNode {
    pub fn new(audio_context: &mut AudioContext) -> Self {
        Self::new_with_signal_value(audio_context, 0.0)
    }

    pub fn new_with_signal_value(audio_context: &mut AudioContext, signal_value: f32) -> Self {
        let new_constant_node = Self {
            uuid: Uuid::new_v4(),
            audio_context: audio_context.clone(),
            signal_value: Rc::new(RefCell::new(signal_value)),
        };

        audio_context.add_node(RefCell::new(Box::new(new_constant_node.clone())));

        new_constant_node
    }

    pub fn signal_value(&self) -> f32 {
        *self.signal_value.borrow()
    }

    pub fn set_signal_value(&mut self, signal_value: f32) -> &mut Self {
        *self.signal_value.borrow_mut() = signal_value;
        self
    }
}

impl Node for ConstantNode {
    fn node_type(&self) -> crate::NodeType {
        NodeType::Input
    }

    fn num_inputs(&self) -> usize {
        0
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn process(&mut self, _: &[Ref<Connection>], outputs: &mut [RefMut<Connection>]) {
        let signal_value = *self.signal_value.borrow();

        // copy to all output connections
        outputs.iter_mut().for_each(|output| {
            output.set_data(signal_value);
            output.set_init(true);
        })
    }

    fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    fn name(&self) -> String {
        String::from("ConstantNode")
    }
}

impl Connect for ConstantNode {
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

impl PartialEq for ConstantNode {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for ConstantNode {}

impl PartialOrd for ConstantNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.uuid.partial_cmp(&other.uuid)
    }
}

impl Ord for ConstantNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uuid.cmp(&other.uuid)
    }
}

impl Hash for ConstantNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}

#[cfg(test)]
mod test_constant_node {
    use std::cell::RefCell;

    use crate::{AudioContext, Connection, ConnectionInner, ConstantNode, Node};

    #[test]
    fn should_output_constant_signal_value() {
        let mut audio_context = AudioContext::new();
        let mut constant_node = ConstantNode::new_with_signal_value(&mut audio_context, 1.234);

        let output_connection = RefCell::new(Connection::from_connection_inner(ConnectionInner {
            from_index: 0,
            to_index: 0,
            data: 0.0,
            init: false,
        }));

        // before processing, output data is 0.0
        {
            let output_connection_ref = output_connection.borrow();
            assert_eq!(output_connection_ref.data(), 0.0);
            assert!(!output_connection_ref.init());
        }

        // run processing for node
        {
            let output_connection_ref_mut = output_connection.borrow_mut();
            let inputs = [];
            let mut outputs = [output_connection_ref_mut];
            constant_node.process(&inputs, &mut outputs)
        }

        // after processing, output data is 1.234
        {
            let output_connection_ref = output_connection.borrow();
            assert_eq!(output_connection_ref.data(), 1.234);
            assert!(output_connection_ref.init());
        }
    }
}
