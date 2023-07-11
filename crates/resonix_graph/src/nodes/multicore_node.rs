use std::{
    any::Any,
    cell::{Ref, RefMut},
};

use resonix_core::NumChannels;

use crate::{Connection, Node, NodeType, NodeUid};

/// Takes many, single-channel connections and combines them into
/// one connection with many channels
#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct MulticoreNode {
    uid: NodeUid,
    num_input_connections: u32,
}

impl MulticoreNode {
    pub fn new(num_input_connections: u32) -> Self {
        Self::new_with_uid(0, num_input_connections)
    }

    pub(crate) fn new_with_uid(uid: NodeUid, num_input_connections: u32) -> Self {
        Self {
            uid,
            num_input_connections,
            ..Default::default()
        }
    }
}

impl Node for MulticoreNode {
    #[inline]
    fn process(
        &mut self,
        inputs: &mut dyn Iterator<Item = &Connection>,
        outputs: &mut dyn Iterator<Item = &mut Connection>,
    ) {
        let mut output = outputs.next().unwrap();
        output
            .data_mut()
            .iter_mut()
            .zip(inputs)
            .for_each(|(output_channel, input_connection)| {
                *output_channel = input_connection.data()[0];
            });
    }

    fn node_type(&self) -> crate::NodeType {
        NodeType::Effect
    }

    fn num_input_connections(&self) -> usize {
        self.num_input_connections as usize
    }

    fn num_output_connections(&self) -> usize {
        1
    }

    fn num_incoming_channels(&self) -> NumChannels {
        // each input channel must be single-channel (for now)
        NumChannels::from(1)
    }

    fn num_outgoing_channels(&self) -> NumChannels {
        NumChannels::from(self.num_input_connections)
    }

    fn uid(&self) -> NodeUid {
        self.uid
    }

    fn set_uid(&mut self, uid: NodeUid) {
        self.uid = uid;
    }
    fn name(&self) -> String {
        String::from("MulticoreNode")
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

    use crate::{Connection, MulticoreNode, Node};

    #[test]
    fn should_combine_many_single_channel_connections_down_to_one_multi_channel_connection() {
        let mut multicore_node = MulticoreNode::new(100);

        let mut input_connections = Vec::new();
        for i in 0..100 {
            input_connections.push(RefCell::new(Connection::from_test_data(
                i,
                1,
                vec![i as f32 / 100.0; 1],
                0,
                0,
            )))
        }
        let output_connection =
            RefCell::new(Connection::from_test_data(1, 100, vec![0.0; 100], 0, 0));

        // before processing, output data is 0.0
        {
            assert_eq!(output_connection.borrow().data(), &vec![0.0; 100]);
        }

        // run processing for node
        {
            let inputs = input_connections.iter().map(|c| c.borrow());
            let outputs = [output_connection.borrow_mut()];
            multicore_node.process(&mut inputs.into_iter(), &mut outputs.into_iter())
        }

        // after processing, outgoing connection should match incoming connections
        {
            let input_connection_data: Vec<f32> = (0..100).map(|i| i as f32 / 100.0).collect();
            assert_eq!(output_connection.borrow().data(), &input_connection_data);
        }
    }
}
