use std::{
    any::Any,
    cell::{Ref, RefMut},
};

use log::info;
use petgraph::prelude::EdgeIndex;
use resonix_core::{NumChannels, SampleRate, Sine, SineInterface};

#[cfg(feature = "dac")]
use {resonix_dac::DACConfig, std::sync::Arc};

use crate::{
    messages::{NodeMessageRequest, NodeMessageResponse},
    Connection, Node, NodeHandle, NodeHandleMessageError, NodeType,
};

#[derive(Debug, Clone)]
pub struct SineNode {
    uid: u32,
    sine: Sine,
    num_outgoing_channels: NumChannels,
    outgoing_connection_indexes: Vec<EdgeIndex>,
}

impl SineNode {
    pub fn new(num_outgoing_channels: impl Into<NumChannels>, frequency: impl Into<f32>) -> Self {
        // sample_rate is automatically configured in the audio thread when "dac" feature is enabled
        Self::new_with_config(num_outgoing_channels, 0, frequency)
    }

    pub fn new_with_config(
        num_outgoing_channels: impl Into<NumChannels>,
        sample_rate: impl Into<SampleRate>,
        frequency: impl Into<f32>,
    ) -> Self {
        Self {
            uid: 0,
            num_outgoing_channels: num_outgoing_channels.into(),
            sine: Sine::new_with_config(sample_rate, frequency),
            outgoing_connection_indexes: Vec::new(),
        }
    }

    #[cfg(test)]
    pub(crate) fn new_with_uid(
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

impl NodeHandle<SineNode> {
    pub async fn set_frequency(&self, new_frequency: f32) -> Result<&Self, NodeHandleMessageError> {
        self.node_request_tx
            .send(NodeMessageRequest::SineSetFrequency {
                node_uid: self.uid,
                node_index: self.node_index,
                new_frequency,
            })
            .await
            .unwrap();

        while let Ok(response) = self.node_response_rx.recv().await {
            let NodeMessageResponse::SineSetFrequency {
                node_uid: uuid,
                result,
            } = response;
            if uuid != self.uid {
                continue;
            }
            return result.map_err(|e| e.into()).map(|_| self);
        }

        Err(NodeHandleMessageError::NoMatchingMessageReceived)
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

    fn uid(&self) -> u32 {
        self.uid
    }

    fn set_uid(&mut self, uid: u32) {
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

    use crate::{AudioContext, Connection, Node, SineNode};

    #[test]
    fn should_output_sine_wave_data() {
        // should finish a sine wave cycle within 4 sample
        let mut sine_node = SineNode::new_with_config(1, 4, 1.0);

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
        let mut sine_node = SineNode::new_with_config(5, 4, 1.0);
        let output_connection = RefCell::new(Connection::from_test_data(0, 5, vec![0.0; 5], 0, 0));

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
