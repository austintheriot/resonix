use std::{
    any::Any,
    sync::{Arc, Mutex},
};

use uuid::Uuid;

use crate::{AudioContext, Connect, ConnectError, Connection, Node, NodeType};

#[derive(Debug, Clone)]
pub struct DACNode {
    data: Arc<Mutex<f32>>,
    uuid: Uuid,
    audio_context: AudioContext,
}

impl DACNode {
    pub fn new(audio_context: &mut AudioContext) -> Self {
        let new_dac_node = Self {
            uuid: Uuid::new_v4(),
            audio_context: audio_context.clone(),
            data: Arc::new(Mutex::new(0.0)),
        };

        audio_context.add_node(new_dac_node.clone());

        new_dac_node
    }

    pub fn data(&self) -> f32 {
        *self.data.lock().unwrap()
    }
}

impl Node for DACNode {
    fn process(
        &mut self,
        inputs: &mut dyn Iterator<Item = Connection>,
        _outputs: &mut dyn Iterator<Item = Connection>,
    ) {
        let Some(first_input) = inputs.next() else {
            return
        };

        let sample = first_input.data();

        *self.data.lock().unwrap() = sample;
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

    fn as_any(&self) -> &dyn Any {
        self
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

impl PartialEq for DACNode {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for DACNode {}

impl PartialOrd for DACNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.uuid.partial_cmp(&other.uuid)
    }
}

impl Ord for DACNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uuid.cmp(&other.uuid)
    }
}

#[cfg(test)]
mod test_dac_node {

    use crate::{AudioContext, Connection, ConnectionInner, DACNode, Node};

    #[test]
    fn should_record_one_sample_of_incoming_data() {
        let mut audio_context = AudioContext::new();
        let mut dac_node = DACNode::new(&mut audio_context);

        let input_connection = Connection::from_connection_inner(ConnectionInner {
            from_index: 0,
            to_index: 0,
            data: 0.1234,
            init: true,
        });

        assert_eq!(dac_node.data(), 0.0);

        {
            let inputs = [input_connection];
            let outputs = [];
            dac_node.process(&mut inputs.into_iter(), &mut outputs.into_iter())
        }

        assert_eq!(dac_node.data(), 0.1234);
    }
}
