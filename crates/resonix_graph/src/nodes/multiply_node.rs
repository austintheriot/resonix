use std::{
    any::Any,
    cell::{Ref, RefMut},
};

use resonix_core::NumChannels;

#[cfg(feature = "dac")]
use {resonix_dac::DACConfig, std::sync::Arc};

use crate::{Connection, Node, NodeType};

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
    num_channels: NumChannels,
}

impl MultiplyNode {
    pub fn new(num_channels: impl Into<NumChannels>) -> Self {
        Self {
            num_channels: num_channels.into(),
            ..Default::default()
        }
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

    fn num_input_connections(&self) -> usize {
        2
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

    #[inline]
    fn process(
        &mut self,
        inputs: &mut dyn Iterator<Item = Ref<Connection>>,
        outputs: &mut dyn Iterator<Item = RefMut<Connection>>,
    ) {
        let first_input = inputs.next().unwrap();
        let second_input = inputs.next().unwrap();

        let mut output = outputs.next().unwrap();

        // todo: parallelize this!
        output.update_data(|frame| {
            frame
                .iter_mut()
                .zip(first_input.data())
                .zip(second_input.data().iter())
                .for_each(|((output, input1), input2)| {
                    *output = *input1 * *input2;
                })
        });
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

    #[cfg(feature = "dac")]
    fn requires_audio_updates(&self) -> bool {
        false
    }

    #[cfg(feature = "dac")]
    fn update_from_dac_config(&mut self, dac_config: Arc<DACConfig>) {}
}

#[cfg(test)]
mod test_multiply_node {

    use std::cell::RefCell;

    use crate::{Connection, MultiplyNode, Node};

    #[test]
    fn should_multiply_1st_and_2nd_inputs() {
        let mut multiply_node = MultiplyNode::new(1);

        let left_input_connection = RefCell::new(Connection::from_test_data(0, 1, vec![0.5], 0, 0));
        let right_input_connection =
            RefCell::new(Connection::from_test_data(1, 1, vec![0.2], 0, 1));
        let output_connection = RefCell::new(Connection::default());

        // before processing, output data is 0.0
        {
            assert_eq!(output_connection.borrow().data(), &vec![0.0]);
        }

        // run processing for node
        {
            let inputs = [
                left_input_connection.borrow(),
                right_input_connection.borrow(),
            ];
            let outputs = [output_connection.borrow_mut()];
            multiply_node.process(&mut inputs.into_iter(), &mut outputs.into_iter())
        }

        // before processing, output data is 0.1
        {
            assert_eq!(output_connection.borrow().data(), &vec![0.1]);
        }
    }

    #[test]
    fn should_work_with_multichannel_data() {
        let left_data = vec![0.0, 2.0, 4.0, 6.0, 8.0];
        let left_input_connection = RefCell::new(Connection::from_test_data(0, 5, left_data, 0, 0));

        let right_data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let right_input_connection =
            RefCell::new(Connection::from_test_data(1, 5, right_data, 0, 1));
        let mut multiply_node = MultiplyNode::new(5);

        let output_connection = RefCell::new(Connection::new(5));

        // before processing, output data is 0.0
        {
            assert_eq!(output_connection.borrow().data(), &vec![0.0; 5]);
        }

        // run processing for node
        {
            let inputs = [
                left_input_connection.borrow(),
                right_input_connection.borrow(),
            ];
            let outputs = [output_connection.borrow_mut()];
            multiply_node.process(&mut inputs.into_iter(), &mut outputs.into_iter())
        }

        // after processing, output data is the result of the multiplication of all channels element-by-element
        {
            assert_eq!(
                output_connection.borrow().data(),
                &vec![0.0, 4.0, 12.0, 24.0, 40.0]
            );
        }
    }
}
