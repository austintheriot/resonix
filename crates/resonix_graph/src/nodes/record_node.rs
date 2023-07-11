use std::{
    any::Any,
    cell::{Ref, RefMut},
};

use resonix_core::NumChannels;

use crate::{Connection, Node, NodeType, NodeUid};

/// Stores data as interleaved buffer of samples
#[derive(Debug, Default, Clone)]
pub struct RecordNode {
    data: Vec<f32>,
    num_incoming_channels: NumChannels,
    uid: NodeUid,
}

impl RecordNode {
    pub fn new(num_incoming_channels: impl Into<NumChannels>) -> Self {
        Self::new_with_uid(0, num_incoming_channels)
    }

    pub(crate) fn new_with_uid(
        uid: NodeUid,
        num_incoming_channels: impl Into<NumChannels>,
    ) -> Self {
        Self {
            uid,
            num_incoming_channels: num_incoming_channels.into(),
            ..Default::default()
        }
    }

    pub fn data(&self) -> &[f32] {
        &self.data
    }
}

impl Node for RecordNode {
    #[inline]
    fn process(
        &mut self,
        inputs: &mut dyn Iterator<Item = Ref<Connection>>,
        _: &mut dyn Iterator<Item = RefMut<Connection>>,
    ) {
        let input = inputs
            .next()
            .expect("RecordNode should have one and only one input connection");
        let input_data = input.data();

        #[cfg(debug_assertions)]
        {
            let input_num_channels = input_data.len();
            let self_num_channels: usize = self.num_incoming_channels.into();
            assert_eq!(input_num_channels, self_num_channels, "Number of channels in the input connection to a RecordNode does not match number of channels that RecordNode was expecting. Expected {self_num_channels} but found {input_num_channels}");
        }

        self.data.extend_from_slice(input_data);
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

    fn uid(&self) -> NodeUid {
        self.uid
    }

    fn set_uid(&mut self, uid: NodeUid) {
        self.uid = uid;
    }
    fn name(&self) -> String {
        String::from("RecordNode")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl PartialEq for RecordNode {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl Eq for RecordNode {}

impl PartialOrd for RecordNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.uid.partial_cmp(&other.uid)
    }
}

impl Ord for RecordNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uid.cmp(&other.uid)
    }
}

#[cfg(test)]
mod test_record_node {

    use std::cell::RefCell;

    use crate::{Connection, Node, RecordNode};

    #[test]
    fn should_record_incoming_node_data() {
        let mut record_node = RecordNode::new(1);

        let input_connection = RefCell::new(Connection::from_test_data(0, 1, vec![0.1234], 0, 0));

        {
            let inputs = [input_connection.borrow()];
            let outputs = [];
            record_node.process(&mut inputs.into_iter(), &mut outputs.into_iter())
        }

        assert_eq!(record_node.data().len(), 1);
        assert_eq!(*record_node.data(), vec![0.1234]);
    }

    #[test]
    fn should_work_with_multichannel_data() {
        let input_connection_data: Vec<f32> = (0..5).map(|i| i as f32).collect();
        let input_connection = RefCell::new(Connection::from_test_data(
            0,
            5,
            input_connection_data.clone(),
            0,
            0,
        ));
        let mut record_node = RecordNode::new(5);

        {
            let inputs = [input_connection.borrow()];
            let outputs = [];
            record_node.process(&mut inputs.into_iter(), &mut outputs.into_iter())
        }

        assert_eq!(record_node.data().len(), 5);
        assert_eq!(*record_node.data(), input_connection_data);
    }
}
