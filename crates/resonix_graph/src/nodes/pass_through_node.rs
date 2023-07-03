use std::{any::Any, cell::{Ref, RefMut}};

use petgraph::prelude::EdgeIndex;
use uuid::Uuid;

use crate::{AddConnectionError, Connection, Node, NodeType};

/// Takes one signal and passed it through, unaltered
/// to all connected outputs.
///
/// Input 0 - Input signal
///
/// Output 0 - Unaltered Input signal
#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct PassThroughNode {
    uid: u32,
}

impl PassThroughNode {
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

impl Node for PassThroughNode {
    #[inline]
    fn process(
        &mut self,
        inputs: &mut dyn Iterator<Item = Ref<Connection>>,
        outputs: &mut dyn Iterator<Item = RefMut<Connection>>,
    ) {
        let input_data = inputs.next().map(|c| c.data()).unwrap_or(0.0);

        // copy first input to all output connections
        for mut output in outputs.into_iter() {
            output.set_data(input_data);
        }
    }

    fn node_type(&self) -> NodeType {
        NodeType::Effect
    }

    fn num_inputs(&self) -> usize {
        1
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn uid(&self) -> u32 {
        self.uid
    }

    fn set_uid(&mut self, uid: u32) {
        self.uid = uid;
    }
    fn name(&self) -> String {
        String::from("PassThroughNode")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}


#[cfg(test)]
mod test_pass_through_node {

    use std::cell::RefCell;

    use crate::{Connection, Node, PassThroughNode};

    #[test]
    fn should_pass_audio_data_through_output_connections() {
        let mut pass_through_node = PassThroughNode::new();

        let input_connection = RefCell::new(Connection::from_test_data(0.1234, 0, 0));

        let mut output_connection = RefCell::new(Connection::default());

        // before processing, output connection holds 0.0
        {
            assert_eq!(output_connection.borrow().data(), 0.0);
        }

        {
            let inputs = [input_connection.borrow()];
            let outputs = [output_connection.borrow_mut()];
            pass_through_node.process(&mut inputs.into_iter(), &mut outputs.into_iter())
        }

        // before processing, output connection holds input data
        {
            assert_eq!(output_connection.borrow().data(), 0.1234);
        }
    }
}
