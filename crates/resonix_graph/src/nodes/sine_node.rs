use std::any::Any;

use resonix_core::{SampleRate, Sine, SineInterface};
use uuid::Uuid;

use crate::{AddToContext, Connection, Node, NodeType};

#[derive(Debug, Clone)]
pub struct SineNode {
    uuid: Uuid,
    sine: Sine,
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
    fn process(
        &mut self,
        _inputs: &mut dyn Iterator<Item = &Connection>,
        outputs: &mut dyn Iterator<Item = &mut Connection>,
    ) {
        let next_sample = self.next_sample();

        outputs.into_iter().for_each(|output| {
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

    fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    fn name(&self) -> String {
        String::from("SineNode")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl SineNode {
    pub fn new() -> Self {
        Self::new_with_config(0, 0.0)
    }

    pub fn new_with_config(sample_rate: impl Into<SampleRate>, frequency: impl Into<f32>) -> Self {
        // todo - get sample rate from audio context

        Self {
            uuid: Uuid::new_v4(),
            sine: Sine::new_with_config(sample_rate, frequency),
        }
    }
}

impl AddToContext for SineNode {}

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

    use uuid::Uuid;

    use crate::{AudioContext, Connection, Node, SineNode};

    #[test]
    fn should_output_sine_wave_data() {
        let _audio_context = AudioContext::new();
        // should finish a sine wave cycle within 4 sample
        let mut sine_node = SineNode::new_with_config(4, 1.0);

        let mut output_connection = Connection {
            from_index: 0,
            to_index: 0,
            data: 0.0,
            uuid: Uuid::new_v4(),
        };

        // before processing, output data is 0.0
        {
            assert_eq!(output_connection.data(), 0.0);
        }

        // run processing for node
        {
            let inputs = [];
            let outputs = [&mut output_connection];
            sine_node.process(&mut inputs.into_iter(), &mut outputs.into_iter());
        }

        // after processing once, output data is 0.0
        {
            assert_eq!(output_connection.data(), 0.0);
        }

        // run processing for node
        {
            let inputs = [];
            let outputs = [&mut output_connection];
            sine_node.process(&mut inputs.into_iter(), &mut outputs.into_iter());
        }

        // after processing twice, output data is 1.0
        {
            assert_eq!(output_connection.data(), 1.0);
        }
    }
}
