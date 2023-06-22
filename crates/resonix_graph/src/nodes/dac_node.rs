use std::any::Any;

use uuid::Uuid;

use crate::{AddToContext, Connection, Node, NodeType};

#[derive(Debug, Clone)]
pub struct DACNode {
    data: f32,
    uuid: Uuid,
}

impl DACNode {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn data(&self) -> f32 {
        self.data
    }
}

impl Node for DACNode {
    fn process(
        &mut self,
        inputs: &mut dyn Iterator<Item = &Connection>,
        _outputs: &mut dyn Iterator<Item = &mut Connection>,
    ) {
        let Some(first_input) = inputs.next() else {
            return
        };

        let sample = first_input.data();

        self.data = sample;
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

impl AddToContext for DACNode {}

impl Default for DACNode {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            data: 0.0,
        }
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

    use uuid::Uuid;

    use crate::{Connection, DACNode, Node};

    #[test]
    fn should_record_one_sample_of_incoming_data() {
        let mut dac_node = DACNode::new();

        let input_connection = Connection {
            from_index: 0,
            to_index: 0,
            data: 0.1234,
            uuid: Uuid::new_v4(),
        };

        assert_eq!(dac_node.data(), 0.0);

        {
            let inputs = [&input_connection];
            let outputs = [];
            dac_node.process(&mut inputs.into_iter(), &mut outputs.into_iter())
        }

        assert_eq!(dac_node.data(), 0.1234);
    }
}
