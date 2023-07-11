use std::{
    any::Any,
    cell::{Ref, RefMut},
};

use petgraph::prelude::EdgeIndex;
use resonix_core::{NumChannels, SampleRate, Sine, SineInterface};

#[cfg(feature = "dac")]
use {resonix_dac::DACConfig, std::sync::Arc};

use crate::{
    messages::{UpdateNodeError, UpdateNodeMessage},
    Connection, Node, NodeType, NodeUid,
};

#[derive(Debug, Clone)]
pub struct SineNode {
    uid: NodeUid,
    sine: Sine,
    num_outgoing_channels: NumChannels,
    outgoing_connection_indexes: Vec<EdgeIndex>,
}

impl SineNode {
    pub fn new(num_outgoing_channels: impl Into<NumChannels>, frequency: impl Into<f32>) -> Self {
        // sample_rate is automatically configured in the audio thread when "dac" feature is enabled
        Self::new_with_uid(0, num_outgoing_channels, frequency)
    }

    pub(crate) fn new_with_uid(
        uid: NodeUid,
        num_outgoing_channels: impl Into<NumChannels>,
        frequency: impl Into<f32>,
    ) -> Self {
        Self::new_with_full_config(uid, num_outgoing_channels, 0, frequency)
    }

    pub(crate) fn new_with_full_config(
        uid: u32,
        num_outgoing_channels: impl Into<NumChannels>,
        sample_rate: impl Into<SampleRate>,
        frequency: impl Into<f32>,
    ) -> Self {
        Self {
            uid,
            num_outgoing_channels: num_outgoing_channels.into(),
            sine: Sine::new_with_config(sample_rate, frequency),
            outgoing_connection_indexes: Vec::new(),
        }
    }
}

impl SineInterface for SineNode {
    fn next_sample(&mut self) -> f32 {
        self.sine.next_sample()
    }

    fn set_sample_rate(&mut self, sample_rate: impl Into<SampleRate>) -> &mut Self {
        self.sine.set_sample_rate(sample_rate);
        self
    }

    fn set_frequency(&mut self, frequency: f32) -> &mut Self {
        self.sine.set_frequency(frequency);
        self
    }

    fn sample_rate(&self) -> SampleRate {
        self.sine.sample_rate()
    }

    fn frequency(&self) -> f32 {
        self.sine.frequency()
    }
}

impl Node for SineNode {
    #[inline]
    fn process(
        &mut self,
        _inputs: &mut dyn Iterator<Item = Ref<Connection>>,
        outputs: &mut dyn Iterator<Item = RefMut<Connection>>,
    ) {
        let next_sample = self.next_sample();

        outputs.into_iter().for_each(|mut output| {
            output.update_data(|buffer| buffer.fill(next_sample));
        });
    }

    fn node_type(&self) -> NodeType {
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

    fn uid(&self) -> NodeUid {
        self.uid
    }

    fn set_uid(&mut self, uid: NodeUid) {
        self.uid = uid;
    }
    fn name(&self) -> String {
        String::from("SineNode")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    #[cfg(feature = "dac")]
    fn requires_audio_updates(&self) -> bool {
        true
    }

    #[cfg(feature = "dac")]
    fn update_from_dac_config(&mut self, dac_config: Arc<DACConfig>) {
        self.sine.set_sample_rate(dac_config.sample_rate());
    }

    #[cfg(feature = "dac")]
    fn handle_update_node_message(
        &mut self,
        update_node_message: UpdateNodeMessage,
    ) -> Result<(), UpdateNodeError> {
        let sine_property = update_node_message.try_into::<SineNodeMessage>()?;

        match sine_property {
            SineNodeMessage::SetFrequency { new_frequency } => {
                self.set_frequency(new_frequency);
            }
        }

        Ok(())
    }
}

pub enum SineNodeMessage {
    SetFrequency { new_frequency: f32 },
}

impl PartialEq for SineNode {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl Eq for SineNode {}

impl PartialOrd for SineNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.uid.partial_cmp(&other.uid)
    }
}

impl Ord for SineNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uid.cmp(&other.uid)
    }
}

#[cfg(test)]
mod test_sine_node {

    use std::cell::RefCell;

    use resonix_core::SineInterface;

    use crate::{messages::UpdateNodeMessage, Connection, Node, SineNode, SineNodeMessage};

    #[cfg(feature = "dac")]
    #[test]
    fn accepts_node_message_request() {
        let update_node_message = UpdateNodeMessage {
            node_uid: 0,
            data: Box::new(SineNodeMessage::SetFrequency {
                new_frequency: 440.0,
            }),
        };

        let mut sine_node = SineNode::new_with_full_config(0, 1, 4, 0.0);

        assert_eq!(sine_node.frequency(), 0.0);

        sine_node
            .handle_update_node_message(update_node_message)
            .unwrap();

        assert_eq!(sine_node.frequency(), 440.0);
    }

    #[test]
    fn should_output_sine_wave_data() {
        // should finish a sine wave cycle within 4 sample
        let mut sine_node = SineNode::new_with_full_config(0, 1, 4, 1.0);

        let output_connection = RefCell::new(Connection::default());

        // before processing, output data is 0.0
        {
            assert_eq!(output_connection.borrow().data(), &vec![0.0]);
        }

        // run processing for node
        {
            let inputs = [];
            let outputs = [output_connection.borrow_mut()];
            sine_node.process(&mut inputs.into_iter(), &mut outputs.into_iter());
        }

        // after processing once, output data is 0.0
        {
            assert_eq!(output_connection.borrow().data(), &vec![0.0]);
        }

        // run processing for node
        {
            let inputs = [];
            let outputs = [output_connection.borrow_mut()];
            sine_node.process(&mut inputs.into_iter(), &mut outputs.into_iter());
        }

        // after processing twice, output data is 1.0
        {
            assert_eq!(output_connection.borrow().data(), &vec![1.0]);
        }
    }

    #[test]
    fn should_work_with_multichannel_data() {
        let mut sine_node = SineNode::new_with_full_config(0, 5, 4, 1.0);
        let output_connection = RefCell::new(Connection::from_test_data(1, 5, vec![0.0; 5], 0, 0));

        // before processing, output data is 0.0
        {
            assert_eq!(output_connection.borrow().data(), &vec![0.0; 5]);
        }

        // run processing for node
        for _ in 0..2 {
            let inputs = [];
            let outputs = [output_connection.borrow_mut()];
            sine_node.process(&mut inputs.into_iter(), &mut outputs.into_iter());
        }

        // after processing twice, output data for all channels should be 1.0
        {
            assert_eq!(output_connection.borrow().data(), &vec![1.0; 5]);
        }
    }
}
