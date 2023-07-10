use std::{
    any::Any,
    cell::{Ref, RefMut},
};

use resonix_core::NumChannels;

use crate::{Connection, Node, NodeType, NodeUid};

#[cfg(feature = "dac")]
use {resonix_dac::DACConfig, std::sync::Arc};

#[derive(Debug, Default, Clone)]
pub struct DACNode {
    data: Vec<f32>,
    num_incoming_channels: NumChannels,
    uid: NodeUid,
}

impl DACNode {
    pub fn new(num_incoming_channels: impl Into<NumChannels>) -> Self {
        Self::new_with_uid(0, num_incoming_channels)
    }

    pub(crate) fn new_with_uid(
        uid: NodeUid,
        num_incoming_channels: impl Into<NumChannels>,
    ) -> Self {
        let num_incoming_channels: NumChannels = num_incoming_channels.into();
        Self {
            uid,
            num_incoming_channels,
            data: vec![0.0; num_incoming_channels.into()],
            ..Default::default()
        }
    }

    pub fn data(&self) -> &Vec<f32> {
        &self.data
    }
}

impl Node for DACNode {
    #[inline]
    fn process(
        &mut self,
        inputs: &mut dyn Iterator<Item = Ref<Connection>>,
        _outputs: &mut dyn Iterator<Item = RefMut<Connection>>,
    ) {
        let first_input = inputs
            .next()
            .expect("DACNode should one and only one input connection");

        let input_frame = first_input.data();

        #[cfg(debug_assertions)]
        {
            let input_num_channels = input_frame.len();
            let self_num_channels: usize = self.num_incoming_channels.into();
            assert_eq!(input_num_channels, self_num_channels, "Number of channels in the input connection to a RecordNode does not match number of channels that RecordNode was expecting. Expected {self_num_channels} but found {input_num_channels}");
        }

        self.data
            .iter_mut()
            .zip(input_frame.iter())
            .for_each(|(output, input)| *output = *input);
    }

    fn node_type(&self) -> NodeType {
        NodeType::Output
    }

    fn num_input_connections(&self) -> usize {
        1
    }

    fn num_output_connections(&self) -> usize {
        0
    }

    fn num_incoming_channels(&self) -> NumChannels {
        self.num_incoming_channels
    }

    fn num_outgoing_channels(&self) -> NumChannels {
        NumChannels::from(0)
    }

    fn uid(&self) -> u32 {
        self.uid
    }

    fn set_uid(&mut self, uid: NodeUid) {
        self.uid = uid;
    }
    fn name(&self) -> String {
        String::from("DACNode")
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

impl PartialEq for DACNode {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl Eq for DACNode {}

impl PartialOrd for DACNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.uid.partial_cmp(&other.uid)
    }
}

impl Ord for DACNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uid.cmp(&other.uid)
    }
}

#[cfg(test)]
mod test_dac_node {

    use std::cell::RefCell;

    use crate::{Connection, DACNode, Node};

    #[test]
    fn should_record_one_sample_of_incoming_data() {
        let mut dac_node = DACNode::new(1);

        let input_connection = RefCell::new(Connection::from_test_data(0, 1, vec![0.1234], 0, 0));

        assert_eq!(dac_node.data(), &vec![0.0]);

        {
            let inputs = [input_connection.borrow()];
            let outputs = [];
            dac_node.process(&mut inputs.into_iter(), &mut outputs.into_iter())
        }

        assert_eq!(dac_node.data(), &vec![0.1234]);
    }

    #[test]
    fn should_work_with_multichannel_data() {
        let mut dac_node = DACNode::new(5);

        let input_data: Vec<f32> = (0..5).map(|i| i as f32).collect();
        let input_connection =
            RefCell::new(Connection::from_test_data(0, 5, input_data.clone(), 0, 0));

        assert_eq!(dac_node.data(), &vec![0.0; 5]);

        {
            let inputs = [input_connection.borrow()];
            let outputs = [];
            dac_node.process(&mut inputs.into_iter(), &mut outputs.into_iter())
        }

        assert_eq!(dac_node.data(), &input_data);
    }
}
