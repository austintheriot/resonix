use std::{
    any::Any,
    cell::{Ref, RefMut},
};

use crate::{Connection, Node, NodeType};

#[derive(Debug, Default, Clone)]
pub struct RecordNode {
    data: Vec<f32>,
    uid: u32,
}

impl RecordNode {
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

    pub fn data(&self) -> &Vec<f32> {
        &self.data
    }
}

impl Node for RecordNode {
    #[inline]
    fn process(
        &mut self,
        inputs: &mut dyn Iterator<Item = Ref<Connection>>,
        _: &mut dyn Iterator<Item = RefMut<Connection>>,
    ) {
        let Some(first_input) = inputs.next() else {
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

    fn uid(&self) -> u32 {
        self.uid
    }

    fn set_uid(&mut self, uid: u32) {
        self.uid = uid;
    }
    fn name(&self) -> String {
        String::from("RecordNode")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl PartialEq for RecordNode {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl Eq for RecordNode {}

impl PartialOrd for RecordNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.uid.partial_cmp(&other.uid)
    }
}

impl Ord for RecordNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uid.cmp(&other.uid)
    }
}

#[cfg(test)]
mod test_record_node {

    use std::cell::RefCell;

    use crate::{Connection, Node, RecordNode};

    #[test]
    fn should_record_incoming_node_data() {
        let mut record_node = RecordNode::new();

        let input_connection = RefCell::new(Connection::from_test_data(0.1234, 0, 0));

        {
            let inputs = [input_connection.borrow()];
            let outputs = [];
            record_node.process(&mut inputs.into_iter(), &mut outputs.into_iter())
        }

        assert_eq!(record_node.data().len(), 1);
        assert_eq!(*record_node.data().first().unwrap(), 0.1234);
    }
}
