use std::{
    any::Any,
    cell::{Ref, RefMut},
    hash::{Hash, Hasher},
};

use resonix_core::{GranularSynthesizer, GranularSynthesizerAction, NumChannels};

#[cfg(feature = "dac")]
use {resonix_dac::DACConfig, std::sync::Arc};

use crate::{
    messages::{MessageError, NodeMessageRequest, UpdateNodeError},
    AudioContext, AudioInit, AudioUninit, Connection, Node, NodeHandle, NodeType, NodeUid,
};

/// Takes no input signals and outputs a single,
/// constant signal value to all output connections.
///
/// Output 0 - Constant signal value
#[derive(Debug, Clone)]
pub struct GranularSynthesizerNode {
    uid: NodeUid,
    num_outgoing_channels: NumChannels,
    granular_synthesizer: GranularSynthesizer,
}

impl GranularSynthesizerNode {
    pub fn new(granular_synthesizer: impl Into<GranularSynthesizer>) -> Self {
       Self::new_with_uid(0, granular_synthesizer)
    }

    pub(crate) fn new_with_uid(
        uid: NodeUid,
        granular_synthesizer: impl Into<GranularSynthesizer>,
    ) -> Self {
        let granular_synthesizer = granular_synthesizer.into();
        // number of outgoing channels is copied here, since the granular 
        // synthesizer is externally read-only it has been converted into a node
        let num_outgoing_channels = granular_synthesizer.num_channels();
        Self {
            uid,
            num_outgoing_channels,
            granular_synthesizer,
        }
    }
}

impl Node for GranularSynthesizerNode {
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
        let mut output = outputs.next().unwrap();
        let output_data = output.data_mut();
        self.granular_synthesizer
            .next_frame_into_buffer(output_data);
    }

    fn uid(&self) -> u32 {
        self.uid
    }

    fn set_uid(&mut self, uid: NodeUid) {
        self.uid = uid;
    }

    fn name(&self) -> String {
        String::from("GranularSynthesizerNode")
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
        self.granular_synthesizer.set_sample_rate(dac_config.sample_rate());
    }
}

impl PartialEq for GranularSynthesizerNode {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl Eq for GranularSynthesizerNode {}

impl PartialOrd for GranularSynthesizerNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.uid.partial_cmp(&other.uid)
    }
}

impl Ord for GranularSynthesizerNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uid.cmp(&other.uid)
    }
}

impl Hash for GranularSynthesizerNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uid.hash(state);
    }
}

#[cfg(test)]
mod test_constant_node {

    use std::{cell::RefCell, sync::Arc};

    use resonix_core::{GranularSynthesizer, GranularSynthesizerAction};

    use crate::{Connection, GranularSynthesizerNode, Node};

    #[test]
    fn should_produce_granular_sounds_from_buffer() {
        let mut granular_synthesizer = GranularSynthesizer::from_seed([0; 32]);
        let buffer_data: Vec<f32> = (0..44100).map(|i| i as f32 / 44100.0).collect();
        let buffer = Arc::new(buffer_data);
        granular_synthesizer.set_buffer(buffer).set_num_channels(1);
        let mut granular_synthesizer_node = GranularSynthesizerNode::new(granular_synthesizer);
        let output_connection = RefCell::new(Connection::from_test_data(1, 1, vec![0.0], 0, 0));

        // before processing, output data is 0.0
        {
            assert_eq!(output_connection.borrow().data(), &vec![0.0]);
        }

        let mut output_buffer = Vec::new();

        // collect single-channel output into a buffer for snapshot testing
        for _ in 0..1000 {
            let inputs = [];
            let outputs = [output_connection.borrow_mut()];
            granular_synthesizer_node.process(&mut inputs.into_iter(), &mut outputs.into_iter());
            let output_connection = output_connection.borrow();
            let mut output_data = output_connection.data().to_owned();
            output_buffer.append(&mut output_data);
        }

        {
            insta::assert_debug_snapshot!(output_buffer);
        }
    }
}
