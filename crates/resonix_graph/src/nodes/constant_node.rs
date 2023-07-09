use std::{
    any::Any,
    cell::{Ref, RefMut},
    hash::{Hash, Hasher},
};

use resonix_core::NumChannels;

#[cfg(feature = "dac")]
use {resonix_dac::DACConfig, std::sync::Arc};

use crate::{Connection, Node, NodeType, NodeUid, NodeHandle, AudioContext, AudioUninit, messages::{UpdateNodeError, NodeMessageRequest, MessageError}, AudioInit};

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

impl NodeHandle<ConstantNode> {
    pub fn set_signal_value_sync(
        &self,
        audio_context: &mut AudioContext<AudioUninit>,
        new_signal_value: f32,
    ) -> Result<&Self, UpdateNodeError> {
        audio_context.handle_node_message_request(NodeMessageRequest::ConstantSetSignalValue {
            node_uid: self.uid,
            new_signal_value,
        })?;

        Ok(self)
    }

    pub async fn set_signal_value_async(
        &self,
        audio_context: &mut AudioContext<AudioInit>,
        new_signal_value: f32,
    ) -> Result<&Self, MessageError> {
        audio_context
            .handle_node_message_request(NodeMessageRequest::ConstantSetSignalValue {
                node_uid: self.uid,
                new_signal_value,
            })
            .await?;

        Ok(self)
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
        _inputs: &mut dyn Iterator<Item = Ref<Connection>>,
        outputs: &mut dyn Iterator<Item = RefMut<Connection>>,
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
    fn requires_audio_updates(&self) -> bool {
        false
    }

    #[cfg(feature = "dac")]
    fn update_from_dac_config(&mut self, _dac_config: Arc<DACConfig>) {}
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
