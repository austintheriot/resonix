use std::{
    any::Any,
    cell::{Ref, RefMut},
};

use crate::{Connection, Node, NodeType};

#[derive(Debug, Default, Clone)]
pub struct DACNode {
    data: f32,
    uid: u32,
}

impl DACNode {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub(crate) fn new_with_uid(uid: u32) -> Self {
        Self {
            uid,
            ..Default::default()
        }
    }

    pub fn data(&self) -> f32 {
        self.data
    }
}

impl Node for DACNode {
    #[inline]
    fn process(
        &mut self,
        inputs: &mut dyn Iterator<Item = Ref<Connection>>,
        _outputs: &mut dyn Iterator<Item = RefMut<Connection>>,
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

    fn uid(&self) -> u32 {
        self.uid
    }

    fn set_uid(&mut self, uid: u32) {
        self.uid = uid;
    }
    fn name(&self) -> String {
        String::from("DACNode")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl PartialEq for DACNode {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl Eq for DACNode {}

impl PartialOrd for DACNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.uid.partial_cmp(&other.uid)
    }
}

impl Ord for DACNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uid.cmp(&other.uid)
    }
}

#[cfg(test)]
mod test_dac_node {

    use std::cell::RefCell;

    use crate::{Connection, DACNode, Node};

    #[test]
    fn should_record_one_sample_of_incoming_data() {
        let mut dac_node = DACNode::new();

        let input_connection = RefCell::new(Connection::from_test_data(0.1234, 0, 0));

        assert_eq!(dac_node.data(), 0.0);

        {
            let inputs = [input_connection.borrow()];
            let outputs = [];
            dac_node.process(&mut inputs.into_iter(), &mut outputs.into_iter())
        }

        assert_eq!(dac_node.data(), 0.1234);
    }
}
