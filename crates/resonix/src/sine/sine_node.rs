use std::{cell::RefCell, rc::Rc, sync::{Arc, Mutex}};

use uuid::Uuid;

use crate::{
    AudioContext, Connect, ConnectError, Connection, Node, NodeType, SampleRate, Sine,
    SineInterface,
};

#[derive(Debug, Clone)]
pub struct SineNode {
    uuid: Uuid,
    sine: Arc<Mutex<Sine>>,
    audio_context: AudioContext,
}

impl SineInterface for SineNode {
    fn next_sample(&mut self) -> f32 {
        self.sine.lock().unwrap().next_sample()
    }

    fn set_sample_rate(&mut self, sample_rate: impl Into<SampleRate>) -> &mut Self {
        self.sine.lock().unwrap().set_sample_rate(sample_rate);
        self
    }

    fn set_frequency(&mut self, frequency: f32) -> &mut Self {
        self.sine.lock().unwrap().set_frequency(frequency);
        self
    }

    fn sample_rate(&self) -> SampleRate {
        self.sine.lock().unwrap().sample_rate()
    }

    fn frequency(&self) -> f32 {
        self.sine.lock().unwrap().frequency()
    }
}

impl Node for SineNode {
    fn process(&mut self, _inputs: &[Connection], outputs: &mut [Connection]) {
        let next_sample = self.next_sample();
        outputs.iter_mut().for_each(|output| {
            output.set_data(next_sample);
            output.set_init(true);
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

    fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    fn name(&self) -> String {
        String::from("SineNode")
    }
}

impl Connect for SineNode {
    fn connect_nodes_with_indexes<N: Node + Connect + Clone>(
        &self,
        from_index: usize,
        other_node: &N,
        to_index: usize,
    ) -> Result<&Self, ConnectError> {
        self.check_index_out_of_bounds(from_index, other_node, to_index)?;

        self.audio_context.connect_nodes_with_indexes(
            self.clone(),
            from_index,
            other_node.clone(),
            to_index,
        );

        Ok(self)
    }
}

impl SineNode {
    pub fn new(audio_context: &mut AudioContext) -> Self {
        Self::new_with_config(audio_context, 0, 0.0)
    }

    pub fn new_with_config(
        audio_context: &mut AudioContext,
        sample_rate: impl Into<SampleRate>,
        frequency: impl Into<f32>,
    ) -> Self {
        // todo - get sample rate from audio context
        let new_sine_node = Self {
            uuid: Uuid::new_v4(),
            sine: Arc::new(Mutex::new(Sine::new_with_config(sample_rate, frequency))),
            audio_context: audio_context.clone(),
        };

        audio_context.add_node(new_sine_node.clone());

        new_sine_node
    }
}

impl PartialEq for SineNode {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for SineNode {}

impl PartialOrd for SineNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.uuid.partial_cmp(&other.uuid)
    }
}

impl Ord for SineNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uuid.cmp(&other.uuid)
    }
}

#[cfg(test)]
mod test_sine_node {

    use crate::{AudioContext, Connection, ConnectionInner, Node, SineNode};

    #[test]
    fn should_output_sine_wave_data() {
        let mut audio_context = AudioContext::new();
        // should finish a sine wave cycle within 4 sample
        let mut sine_node = SineNode::new_with_config(&mut audio_context, 4, 1.0);

        let mut output_connection = Connection::from_connection_inner(ConnectionInner {
            from_index: 0,
            to_index: 0,
            data: 0.0,
            init: false,
        });

        // before processing, output data is 0.0
        {
            assert_eq!(output_connection.data(), 0.0);
            assert!(!output_connection.init());
        }

        // run processing for node
        {
            let inputs = [];
            let mut outputs = [output_connection.clone()];
            sine_node.process(&inputs, &mut outputs);
        }

        // after processing once, output data is 0.0
        {
            assert_eq!(output_connection.data(), 0.0);
            assert!(output_connection.init());
        }

        // run processing for node
        {
            output_connection.set_init(false);
            let inputs = [];
            let mut outputs = [output_connection.clone()];
            sine_node.process(&inputs, &mut outputs);
        }

        // after processing twice, output data is 1.0
        {
            assert_eq!(output_connection.data(), 1.0);
            assert!(output_connection.init());
        }
    }
}
