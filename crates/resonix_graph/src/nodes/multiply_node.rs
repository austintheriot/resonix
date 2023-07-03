use std::{any::Any, cell::{Ref, RefMut}};

use petgraph::prelude::EdgeIndex;
use uuid::Uuid;

use crate::{AddConnectionError, Connection, Node, NodeType};

/// Takes two signals and multiplies them together,
/// outputting the signal to all connected outputs
///
/// Input 0 - Signal 1
/// Input 1 - Signal 2
///
/// Output 0 - Multiplied signal
#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct MultiplyNode {
    uid: u32,
}

impl MultiplyNode {
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
}

impl Node for MultiplyNode {
    fn node_type(&self) -> crate::NodeType {
        NodeType::Effect
    }

    fn num_inputs(&self) -> usize {
        2
    }

    fn num_outputs(&self) -> usize {
        1
    }

    #[inline]
    fn process(
        &mut self,
        inputs: &mut dyn Iterator<Item = Ref<Connection>>,
        outputs: &mut dyn Iterator<Item = RefMut<Connection>>,
    ) {
        let first_input = inputs.next().unwrap();
        let second_input = inputs.next().unwrap();
        let result = first_input.data() * second_input.data();

        // copy to all output connections
        outputs.into_iter().for_each(|mut output| {
            output.set_data(result);
        })
    }

    fn uid(&self) -> u32 {
        self.uid
    }

    fn set_uid(&mut self, uid: u32) {
        self.uid = uid;
    }
    fn name(&self) -> String {
        String::from("MultiplyNode")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod test_multiply_node {

    use std::cell::RefCell;

    use crate::{Connection, MultiplyNode, Node};

    #[test]
    fn should_multiply_1st_and_2nd_inputs() {
        let mut multiply_node = MultiplyNode::new();

        let left_input_connection = RefCell::new(Connection::from_test_data(0.5, 0, 0));
        let right_input_connection = RefCell::new(Connection::from_test_data(0.2, 0, 1));
        let mut output_connection = RefCell::new(Connection::default());

        // before processing, output data is 0.0
        {
            assert_eq!(output_connection.borrow().data(), 0.0);
        }

        // run processing for node
        {
            let inputs = [left_input_connection.borrow(), right_input_connection.borrow()];
            let outputs = [output_connection.borrow_mut()];
            multiply_node.process(&mut inputs.into_iter(), &mut outputs.into_iter())
        }

        // before processing, output data is 0.1
        {
            assert_eq!(output_connection.borrow().data(), 0.1);
        }
    }
}
