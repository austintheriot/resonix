use std::{
    any::Any,
    hash::{Hash, Hasher},
    sync::{Arc, Mutex},
};

use uuid::Uuid;

use crate::{AudioContext, Connect, ConnectError, Connection, Node, NodeType};

/// Takes no input signals and outputs a single,
/// constant signal value to all output connections.
///
/// Output 0 - Constant signal value
#[derive(Debug, Clone)]
pub struct ConstantNode {
    uuid: Uuid,
    audio_context: AudioContext,
    signal_value: Arc<Mutex<f32>>,
}

impl ConstantNode {
    pub fn new(audio_context: &mut AudioContext) -> Self {
        Self::new_with_signal_value(audio_context, 0.0)
    }

    pub fn new_with_signal_value(audio_context: &mut AudioContext, signal_value: f32) -> Self {
        let new_constant_node = Self {
            uuid: Uuid::new_v4(),
            audio_context: audio_context.clone(),
            signal_value: Arc::new(Mutex::new(signal_value)),
        };

        audio_context.add_node(new_constant_node.clone());

        new_constant_node
    }

    pub fn signal_value(&self) -> f32 {
        *self.signal_value.lock().unwrap()
    }

    pub fn set_signal_value(&mut self, signal_value: f32) -> &mut Self {
        *self.signal_value.lock().unwrap() = signal_value;
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

    fn process(
        &mut self,
        _: &mut dyn Iterator<Item = Connection>,
        outputs: &mut dyn Iterator<Item = Connection>,
    ) {
        let signal_value = *self.signal_value.lock().unwrap();

        // copy to all output connections
        outputs.into_iter().for_each(|mut output| {
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Connect for ConstantNode {
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

    use crate::{AudioContext, Connection, ConnectionInner, ConstantNode, Node};

    #[test]
    fn should_output_constant_signal_value() {
        let mut audio_context = AudioContext::new();
        let mut constant_node = ConstantNode::new_with_signal_value(&mut audio_context, 1.234);

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
            let inputs = [];
            let outputs = [output_connection.clone()];
            constant_node.process(&mut inputs.into_iter(), &mut outputs.into_iter())
        }

        // after processing, output data is 1.234
        {
            assert_eq!(output_connection.data(), 1.234);
            assert!(output_connection.init());
        }
    }
}
