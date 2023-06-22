use std::any::Any;

use uuid::Uuid;

use crate::{AddToContext, AudioContext, Connection, Node, NodeType};

#[derive(Debug, Clone)]
pub struct RecordNode {
    data: Vec<f32>,
    uuid: Uuid,
}

impl RecordNode {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn data(&self) -> &Vec<f32> {
        &self.data
    }
}

impl Node for RecordNode {
    fn process(&mut self, inputs: &[&Connection], _: &mut [&mut Connection]) {
        let Some(first_input) = inputs.first() else {
            return
        };

        self.data.push(first_input.data());
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl AddToContext for RecordNode {}

impl Default for RecordNode {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            data: Vec::new(),
        }
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

    use uuid::Uuid;

    use crate::{Connection, Node, RecordNode};

    #[test]
    fn should_record_incoming_node_data() {
        let mut record_node = RecordNode::new();

        let input_connection = Connection {
            from_index: 0,
            to_index: 0,
            data: 0.1234,
            uuid: Uuid::new_v4(),
        };

        {
            let inputs = [&input_connection];
            let mut outputs = [];
            record_node.process(&inputs, &mut outputs)
        }

        assert_eq!(record_node.data().len(), 1);
        assert_eq!(*record_node.data().first().unwrap(), 0.1234);
    }
}
