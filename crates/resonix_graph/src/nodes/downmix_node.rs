use std::{
    any::Any,
    cell::{Ref, RefMut},
};

use resonix_core::{Downmixer, NumChannels};

use crate::{Connection, Node, NodeType, NodeUid};

/// Takes a multichannel signal with number of channels `m`
/// and downmixes the output to a multichannel signal with number
/// of channels `n`, where `n <= m`
#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct DownmixNode {
    uid: NodeUid,
    num_incoming_channels: NumChannels,
    num_outgoing_channels: NumChannels,
    downmixer: Downmixer,
}

impl DownmixNode {
    pub fn new(
        num_incoming_channels: impl Into<NumChannels>,
        num_outgoing_channels: impl Into<NumChannels>,
        downmixer: Downmixer,
    ) -> Self {
        Self::new_with_uid(0, num_incoming_channels, num_outgoing_channels, downmixer)
    }

    pub(crate) fn new_with_uid(
        uid: NodeUid,
        num_incoming_channels: impl Into<NumChannels>,
        num_outgoing_channels: impl Into<NumChannels>,
        downmixer: Downmixer,
    ) -> Self {
        Self {
            uid,
            num_incoming_channels: num_incoming_channels.into(),
            num_outgoing_channels: num_outgoing_channels.into(),
            downmixer,
            ..Default::default()
        }
    }
}

impl Node for DownmixNode {
    #[inline]
    fn process(
        &mut self,
        inputs: &mut dyn Iterator<Item = Ref<Connection>>,
        outputs: &mut dyn Iterator<Item = RefMut<Connection>>,
    ) {
        let input = inputs.next().unwrap();
        let input_data = input.data();

        let mut output = outputs.next().unwrap();
        let output_data_mut = output.data_mut();

        self.downmixer.as_downmixer_to_buffer()(
            input_data,
            output_data_mut.len() as u32,
            output_data_mut,
        );
    }

    fn node_type(&self) -> crate::NodeType {
        NodeType::Effect
    }

    fn num_input_connections(&self) -> usize {
        1
    }

    fn num_output_connections(&self) -> usize {
        1
    }

    fn num_incoming_channels(&self) -> NumChannels {
        self.num_incoming_channels
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
        String::from("DownmixNode")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod test_multiply_node {

    use std::{cell::RefCell, vec};

    use resonix_core::Downmixer;

    use crate::{Connection, DownmixNode, Node};

    #[test]
    fn should_downmix_multichannel_data() {
        let mut downmix_node = DownmixNode::new(10, 2, Downmixer::Panning);

        let input_connection_data: Vec<f32> = (0..10).map(|i| i as f32 / 10.0).collect();
        let input_connection = RefCell::new(Connection::from_test_data(
            0,
            10,
            input_connection_data,
            0,
            0,
        ));
        let output_connection = RefCell::new(Connection::from_test_data(1, 2, vec![0.0; 2], 0, 0));

        // before processing, output data is 0.0
        {
            assert_eq!(output_connection.borrow().data(), &vec![0.0; 2]);
        }

        // run processing for node
        {
            let inputs = [input_connection.borrow()];
            let outputs = [output_connection.borrow_mut()];
            downmix_node.process(&mut inputs.into_iter(), &mut outputs.into_iter())
        }

        // compare downmixed snapshot
        {
            insta::assert_debug_snapshot!(output_connection.borrow().data());
        }
    }
}
