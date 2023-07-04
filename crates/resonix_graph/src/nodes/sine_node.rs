use std::{
    any::Any,
    cell::{Ref, RefMut},
};

use log::info;
use petgraph::prelude::EdgeIndex;
use resonix_core::{SampleRate, Sine, SineInterface};

use crate::{
    messages::{NodeMessageRequest, NodeMessageResponse},
    Connection, Node, NodeHandle, NodeHandleMessageError, NodeType,
};

#[derive(Debug, Clone)]
pub struct SineNode {
    uid: u32,
    sine: Sine,
    outgoing_connection_indexes: Vec<EdgeIndex>,
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
            info!("sine_node message received!: {uuid:?}, {result:?}");
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
            output.set_data(next_sample);
        });
    }

    fn node_type(&self) -> NodeType {
        NodeType::Input
    }

    fn num_inputs(&self) -> usize {
        0
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
        String::from("SineNode")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl SineNode {
    pub fn new() -> Self {
        Self::new_with_config(0, 0.0)
    }

    pub fn new_with_config(sample_rate: impl Into<SampleRate>, frequency: impl Into<f32>) -> Self {
        // todo - get sample rate from audio context by default

        Self {
            uid: 0,
            sine: Sine::new_with_config(sample_rate, frequency),
            outgoing_connection_indexes: Vec::new(),
        }
    }

    #[cfg(test)]
    pub(crate) fn new_with_uid(
        uid: u32,
        sample_rate: impl Into<SampleRate>,
        frequency: impl Into<f32>,
    ) -> Self {
        Self {
            uid,
            sine: Sine::new_with_config(sample_rate, frequency),
            outgoing_connection_indexes: Vec::new(),
        }
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
        let _audio_context = AudioContext::new();
        // should finish a sine wave cycle within 4 sample
        let mut sine_node = SineNode::new_with_config(4, 1.0);

        let output_connection = RefCell::new(Connection::default());

        // before processing, output data is 0.0
        {
            assert_eq!(output_connection.borrow().data(), 0.0);
        }

        // run processing for node
        {
            let inputs = [];
            let outputs = [output_connection.borrow_mut()];
            sine_node.process(&mut inputs.into_iter(), &mut outputs.into_iter());
        }

        // after processing once, output data is 0.0
        {
            assert_eq!(output_connection.borrow().data(), 0.0);
        }

        // run processing for node
        {
            let inputs = [];
            let outputs = [output_connection.borrow_mut()];
            sine_node.process(&mut inputs.into_iter(), &mut outputs.into_iter());
        }

        // after processing twice, output data is 1.0
        {
            assert_eq!(output_connection.borrow().data(), 1.0);
        }
    }
}
