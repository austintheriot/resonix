use std::{
    any::Any,
    cell::{Ref, RefMut},
    vec,
};

use resonix_core::NumChannels;

use crate::{Connection, Node, NodeType};

/// Takes one signal and passed it through, unaltered
/// to all connected outputs.
///
/// Input 0 - Input signal
///
/// Output 0 - Unaltered Input signal
#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct PassThroughNode {
    uid: u32,
    num_channels: NumChannels,
}

impl PassThroughNode {
    pub fn new(num_channels: impl Into<NumChannels>) -> Self {
        Self {
            num_channels: num_channels.into(),
            ..Default::default()
        }
    }

    #[cfg(test)]
    pub(crate) fn new_with_uid(uid: u32, num_channels: impl Into<NumChannels>) -> Self {
        Self {
            uid,
            num_channels: num_channels.into(),
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
        // it's possible for a pass through node to be created that hasn't been
        // connected to an outgoing connection yet, so this shouldn't cause an error
        if let Some(mut output) = outputs.next() {
            let input = inputs
                .next()
                .expect("PassThrough node should have one and only one input connection");
            let input_data = input.data();

            output.update_data(|frame| {
                frame
                    .iter_mut()
                    .zip(input_data.iter())
                    .for_each(|(output, input)| {
                        *output = *input;
                    })
            });
        }
    }

    fn node_type(&self) -> NodeType {
        NodeType::Effect
    }

    fn num_input_connections(&self) -> usize {
        1
    }

    fn num_output_connections(&self) -> usize {
        1
    }

    fn num_incoming_channels(&self) -> NumChannels {
        self.num_channels
    }

    fn num_outgoing_channels(&self) -> NumChannels {
        self.num_channels
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
        let mut pass_through_node = PassThroughNode::new(1);

        let input_connection = RefCell::new(Connection::from_test_data(0, 1, vec![0.1234], 0, 0));

        let output_connection = RefCell::new(Connection::default());

        // before processing, output connection holds 0.0
        {
            assert_eq!(output_connection.borrow().data(), &vec![0.0]);
        }

        {
            let inputs = [input_connection.borrow()];
            let outputs = [output_connection.borrow_mut()];
            pass_through_node.process(&mut inputs.into_iter(), &mut outputs.into_iter())
        }

        // before processing, output connection holds input data
        {
            assert_eq!(output_connection.borrow().data(), &vec![0.1234]);
        }
    }

    #[test]
    fn should_work_with_multichannel_data() {
        let input_connection_data: Vec<f32> = (0..5).map(|i| i as f32).collect();
        let input_connection = RefCell::new(Connection::from_test_data(
            0,
            5,
            input_connection_data.clone(),
            0,
            0,
        ));

        let mut pass_through_node = PassThroughNode::new(5);

        let output_connection = RefCell::new(Connection::new(5));

        // before processing, output connection holds 0.0
        {
            assert_eq!(output_connection.borrow().data(), &vec![0.0; 5]);
        }

        {
            let inputs = [input_connection.borrow()];
            let outputs = [output_connection.borrow_mut()];
            pass_through_node.process(&mut inputs.into_iter(), &mut outputs.into_iter())
        }

        // before processing, output connection holds input data
        {
            assert_eq!(output_connection.borrow().data(), &input_connection_data);
        }
    }
}
