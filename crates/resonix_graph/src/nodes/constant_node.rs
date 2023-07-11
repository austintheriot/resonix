use std::{
    any::Any,
    cell::{Ref, RefMut},
    hash::{Hash, Hasher},
};

use resonix_core::NumChannels;

use crate::{
    messages::{UpdateNodeError, UpdateNodeMessage},
    Connection, Node, NodeType, NodeUid,
};

/// Takes no input signals and outputs a single,
/// constant signal value to all output connections.
///
/// Output 0 - Constant signal value
#[derive(Debug, Clone)]
pub struct ConstantNode {
    uid: NodeUid,
    num_outgoing_channels: NumChannels,
    signal_value: f32,
}

impl ConstantNode {
    pub fn new(num_outgoing_channels: impl Into<NumChannels>, signal_value: f32) -> Self {
        Self::new_with_uid(0, num_outgoing_channels, signal_value)
    }

    pub(crate) fn new_with_uid(
        uid: NodeUid,
        num_outgoing_channels: impl Into<NumChannels>,
        signal_value: f32,
    ) -> Self {
        Self {
            uid,
            num_outgoing_channels: num_outgoing_channels.into(),
            signal_value,
        }
    }

    pub fn signal_value(&self) -> f32 {
        self.signal_value
    }

    pub fn set_signal_value(&mut self, signal_value: f32) -> &mut Self {
        self.signal_value = signal_value;
        self
    }
}

impl Node for ConstantNode {
    fn node_type(&self) -> crate::NodeType {
        NodeType::Input
    }

    fn num_input_connections(&self) -> usize {
        0
    }

    fn num_output_connections(&self) -> usize {
        1
    }

    fn num_incoming_channels(&self) -> NumChannels {
        NumChannels::from(0)
    }

    fn num_outgoing_channels(&self) -> NumChannels {
        self.num_outgoing_channels
    }

    #[inline]
    fn process(
        &mut self,
        _inputs: &mut dyn Iterator<Item = &Connection>,
        outputs: &mut dyn Iterator<Item = &mut Connection>,
    ) {
        // copy to all output connections
        outputs.into_iter().for_each(|mut output| {
            output.update_data(|values| values.fill(self.signal_value));
        })
    }

    fn uid(&self) -> u32 {
        self.uid
    }

    fn set_uid(&mut self, uid: NodeUid) {
        self.uid = uid;
    }

    fn name(&self) -> String {
        String::from("ConstantNode")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    #[cfg(feature = "dac")]
    fn handle_update_node_message(
        &mut self,
        update_node_message: UpdateNodeMessage,
    ) -> Result<(), UpdateNodeError> {
        let sine_property = update_node_message.try_into::<ConstantNodeMessage>()?;

        match sine_property {
            ConstantNodeMessage::SetSignalValue { new_signal_value } => {
                self.set_signal_value(new_signal_value);
            }
        }

        Ok(())
    }
}

pub enum ConstantNodeMessage {
    SetSignalValue { new_signal_value: f32 },
}

impl PartialEq for ConstantNode {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl Eq for ConstantNode {}

impl PartialOrd for ConstantNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.uid.partial_cmp(&other.uid)
    }
}

impl Ord for ConstantNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uid.cmp(&other.uid)
    }
}

impl Hash for ConstantNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uid.hash(state);
    }
}

#[cfg(test)]
mod test_constant_node {

    use std::cell::RefCell;

    use crate::{Connection, ConstantNode, Node};

    #[cfg(feature = "dac")]
    #[test]
    fn accepts_node_message_request() {
        use crate::{messages::UpdateNodeMessage, ConstantNodeMessage};

        let update_node_message = UpdateNodeMessage {
            node_uid: 0,
            data: Box::new(ConstantNodeMessage::SetSignalValue {
                new_signal_value: 1.0,
            }),
        };

        let mut constant_node = ConstantNode::new(1, 0.0);

        assert_eq!(constant_node.signal_value(), 0.0);

        constant_node
            .handle_update_node_message(update_node_message)
            .unwrap();

        assert_eq!(constant_node.signal_value(), 1.0);
    }

    #[cfg(feature = "dac")]
    #[test]
    fn rejects_invalid_node_message_request() {
        use crate::{
            messages::{UpdateNodeError, UpdateNodeMessage},
            SineNodeMessage,
        };

        let update_node_message = UpdateNodeMessage {
            node_uid: 0,
            data: Box::new(SineNodeMessage::SetFrequency {
                new_frequency: 440.0,
            }),
        };

        let mut constant_node = ConstantNode::new(1, 0.0);

        let result = constant_node.handle_update_node_message(update_node_message);

        assert!(matches!(
            result,
            Err(UpdateNodeError::InvalidData { uid: 0 })
        ))
    }

    #[test]
    fn should_output_constant_signal_value() {
        let mut constant_node = ConstantNode::new(1, 1.234);

        let output_connection = RefCell::new(Connection::default());

        // before processing, output data is 0.0
        {
            assert_eq!(output_connection.borrow().data(), &vec![0.0]);
        }

        // run processing for node
        {
            let inputs = [];
            let outputs = [output_connection.borrow_mut()];
            constant_node.process(&mut inputs.into_iter(), &mut outputs.into_iter())
        }

        // after processing, output data is 1.234
        {
            assert_eq!(output_connection.borrow().data(), &vec![1.234]);
        }
    }

    #[test]
    fn should_work_with_multichannel_data() {
        let mut constant_node = ConstantNode::new(5, 0.5);

        let output_connection = RefCell::new(Connection::new(5));

        // before processing, output data is 0.0
        {
            assert_eq!(output_connection.borrow().data(), &vec![0.0; 5]);
        }

        // run processing for node
        {
            let inputs = [];
            let outputs = [output_connection.borrow_mut()];
            constant_node.process(&mut inputs.into_iter(), &mut outputs.into_iter())
        }

        // after processing, output data is 1.234
        {
            assert_eq!(output_connection.borrow().data(), &vec![0.5; 5]);
        }
    }
}
