use std::{
    cell::{Ref, RefCell},
    rc::Rc, sync::{Arc, Mutex, MutexGuard},
};

use uuid::Uuid;

use crate::{AudioContext, Connect, ConnectError, Connection, Node, NodeType};

#[derive(Debug, Clone)]
pub struct RecordNode {
    data: Arc<Mutex<Vec<f32>>>,
    uuid: Uuid,
    audio_context: AudioContext,
}

impl RecordNode {
    pub fn new(audio_context: &mut AudioContext) -> Self {
        let new_record_node = Self {
            uuid: Uuid::new_v4(),
            audio_context: audio_context.clone(),
            data: Arc::new(Mutex::new(Vec::new())),
        };

        audio_context.add_node(new_record_node.clone());

        new_record_node
    }

    pub fn data(&self) -> MutexGuard<Vec<f32>> {
        self.data.lock().unwrap()
    }
}

impl Node for RecordNode {
    fn process(&mut self, inputs: &[Connection], _outputs: &mut [Connection]) {
        let Some(first_input) = inputs.first() else {
            return
        };

        self.data.lock().unwrap().push(first_input.data());
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
        String::from("RecordNode")
    }
}

impl Connect for RecordNode {
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

impl PartialEq for RecordNode {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for RecordNode {}

impl PartialOrd for RecordNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.uuid.partial_cmp(&other.uuid)
    }
}

impl Ord for RecordNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uuid.cmp(&other.uuid)
    }
}

#[cfg(test)]
mod test_record_node {

    use crate::{AudioContext, Connection, ConnectionInner, Node, RecordNode};

    #[test]
    fn should_record_incoming_node_data() {
        let mut audio_context = AudioContext::new();
        let mut record_node = RecordNode::new(&mut audio_context);

        let input_connection = Connection::from_connection_inner(ConnectionInner {
            from_index: 0,
            to_index: 0,
            data: 0.1234,
            init: true,
        });

        {
            let inputs = [input_connection];
            let mut outputs = [];
            record_node.process(&inputs, &mut outputs)
        }

        assert_eq!(record_node.data().len(), 1);
        assert_eq!(*record_node.data().first().unwrap(), 0.1234);
    }
}
