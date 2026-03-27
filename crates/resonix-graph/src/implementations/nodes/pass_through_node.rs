use core::ops::Deref;

use crate::{
    errors::AudioNodeRunError,
    primitives::{
        BlockSize, CurrentTime, Id, NodeId, PortAddress, PortAddressDirection, PortDescriptor,
        PortId, Priority,
    },
    traits::{AudioNode, DescribePorts, GenerateId, GetNodeId, GetPortDescriptors, GetPriority},
};

/// A node with one internal input and one internal output that copies
/// the graph-supplied input buffer to the graph-supplied output buffer.
pub struct PassthroughNode {
    node_id: NodeId,
    port_descriptors: PassthroughPortDescriptors,
}

impl PassthroughNode {
    pub fn new<G: GenerateId>(id_gen: &mut G) -> Self {
        let default_channels = 1;
        PassthroughNode::new_with_channels(id_gen, default_channels)
    }

    pub fn new_with_channels<G: GenerateId>(id_gen: &mut G, channels: usize) -> Self {
        let node_id = NodeId::from(id_gen.generate_id());
        Self {
            node_id,
            port_descriptors: PassthroughPortDescriptors::new(node_id, channels),
        }
    }
}

impl GetNodeId for PassthroughNode {
    fn node_id(&self) -> Id {
        *self.node_id
    }
}

impl GetPriority for PassthroughNode {
    // TODO: implement real priority at some point, use id for now
    fn get_priority(&self) -> Priority {
        (**self.node_id).into()
    }
}

impl Deref for PassthroughNode {
    type Target = PassthroughPortDescriptors;
    fn deref(&self) -> &Self::Target {
        &self.port_descriptors
    }
}

impl AudioNode for PassthroughNode {
    fn process<A: crate::traits::AudioBuffer, M: crate::traits::AudioBufferMut>(
        &mut self,
        inputs: &[Option<A>],
        outputs: &mut [Option<M>],
        _block_size: BlockSize,
        _current_time: CurrentTime,
    ) -> Result<(), AudioNodeRunError> {
        // no output: nothing to write
        let Some(output_buffer) = outputs
            .get_mut(**PassthroughPortDescriptors::OUTPUT_PORT_ID)
            .and_then(|o| o.as_mut())
        else {
            return Ok(());
        };

        // no input: nothing to write
        let Some(input_buffer) = inputs
            .get(**PassthroughPortDescriptors::INPUT_PORT_ID)
            .and_then(|i| i.as_ref())
        else {
            return Ok(());
        };

        // copy input channel samples to output channel samples
        for (channel_i, channel) in input_buffer.channels_iter().unwrap().enumerate() {
            for (sample_i, sample) in channel.iter().enumerate() {
                // channels matching checked at `connect` time
                let output_channel = output_buffer.channel_mut(channel_i).unwrap();
                // channel block_size is guaranteed by the Graph implementation
                output_channel[sample_i] = *sample;
            }
        }

        Ok(())
    }
}

#[derive(Copy, Clone)]
pub struct PassthroughPortDescriptors {
    input_port_descriptor: [PortDescriptor; 1],
    output_port_descriptor: [PortDescriptor; 1],
}

impl PassthroughPortDescriptors {
    pub const INPUT_PORT_ID: PortId = PortId::new(0);
    pub const OUTPUT_PORT_ID: PortId = PortId::new(0);

    pub const fn new(node_id: NodeId, channels: usize) -> Self {
        Self {
            input_port_descriptor: [PortDescriptor {
                address: PortAddress::new(
                    node_id,
                    Self::INPUT_PORT_ID,
                    PortAddressDirection::Input,
                ),
                channels,
            }],
            output_port_descriptor: [PortDescriptor {
                address: PortAddress::new(
                    node_id,
                    Self::OUTPUT_PORT_ID,
                    PortAddressDirection::Output,
                ),
                channels,
            }],
        }
    }
}

impl DescribePorts for PassthroughPortDescriptors {
    fn input_ports(&self) -> Option<&[PortDescriptor]> {
        Some(&self.input_port_descriptor)
    }

    fn output_ports(&self) -> Option<&[PortDescriptor]> {
        Some(&self.output_port_descriptor)
    }
}

impl GetPortDescriptors<PassthroughPortDescriptors> for PassthroughNode {
    fn get_port_descriptors(&self) -> PassthroughPortDescriptors {
        self.port_descriptors
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::*;
    use crate::{
        primitives::Sample,
        test_utils::{InputBufferKeyMapping, OutputBufferKeyMapping, TestIdGenerator, run_process},
    };

    #[test]
    fn it_should_copy_single_channel_input_to_output() {
        let mut id_gen = TestIdGenerator(0);
        let num_channels = 1;
        let mut node = PassthroughNode::new_with_channels(&mut id_gen, num_channels);

        let raw_input_buffer: Vec<Sample> =
            [0.0, 1.0, 2.0, 3.0].into_iter().map(Sample::from).collect();
        let mut raw_output_buffer: Vec<Sample> = [0.0; 4].into_iter().map(Sample::from).collect();
        let block_size = raw_input_buffer.len();

        run_process(
            &mut node,
            Some(&[InputBufferKeyMapping {
                id: PassthroughPortDescriptors::INPUT_PORT_ID,
                buffer: raw_input_buffer.as_slice(),
                num_channels,
            }]),
            Some(&mut [OutputBufferKeyMapping {
                id: PassthroughPortDescriptors::OUTPUT_PORT_ID,
                buffer: Some(raw_output_buffer.as_mut_slice()),
                num_channels,
            }]),
            block_size,
            CurrentTime::from(0.0),
        )
        .unwrap();

        assert_eq!(raw_input_buffer, raw_output_buffer);
    }

    #[test]
    fn it_should_copy_multi_channel_input_to_output() {
        let mut id_gen = TestIdGenerator(0);
        let num_channels = 4;
        let mut node = PassthroughNode::new_with_channels(&mut id_gen, num_channels);

        let raw_input_buffer: Vec<Sample> =
            [0.0, 1.0, 2.0, 3.0].into_iter().map(Sample::from).collect();
        let mut raw_output_buffer: Vec<Sample> = [0.0; 4].into_iter().map(Sample::from).collect();
        let block_size = raw_input_buffer.len();

        run_process(
            &mut node,
            Some(&[InputBufferKeyMapping {
                id: PassthroughPortDescriptors::INPUT_PORT_ID,
                buffer: raw_input_buffer.as_slice(),
                num_channels,
            }]),
            Some(&mut [OutputBufferKeyMapping {
                id: PassthroughPortDescriptors::OUTPUT_PORT_ID,
                buffer: Some(raw_output_buffer.as_mut_slice()),
                num_channels,
            }]),
            block_size,
            CurrentTime::from(0.0),
        )
        .unwrap();

        assert_eq!(raw_input_buffer, raw_output_buffer);
    }

    #[test]
    fn it_should_not_panic_on_empty_inputs() {
        let mut id_gen = TestIdGenerator(0);
        let num_channels = 4;
        let mut node = PassthroughNode::new_with_channels(&mut id_gen, num_channels);

        let mut raw_output_buffer: Vec<Sample> = [0.0; 4].into_iter().map(Sample::from).collect();
        let expected_raw_output_buffer = raw_output_buffer.clone();
        let block_size = raw_output_buffer.len();

        run_process(
            &mut node,
            None,
            Some(&mut [OutputBufferKeyMapping {
                id: PassthroughPortDescriptors::OUTPUT_PORT_ID,
                buffer: Some(raw_output_buffer.as_mut_slice()),
                num_channels,
            }]),
            block_size,
            CurrentTime::from(0.0),
        )
        .unwrap();

        assert_eq!(expected_raw_output_buffer, raw_output_buffer);
    }

    #[test]
    fn it_should_not_panic_on_empty_outputs() {
        let mut id_gen = TestIdGenerator(0);
        let num_channels = 4;
        let mut node = PassthroughNode::new_with_channels(&mut id_gen, num_channels);

        let raw_input_buffer: Vec<Sample> =
            [0.0, 1.0, 2.0, 3.0].into_iter().map(Sample::from).collect();
        let block_size = raw_input_buffer.len();

        run_process(
            &mut node,
            Some(&[InputBufferKeyMapping {
                id: PassthroughPortDescriptors::INPUT_PORT_ID,
                buffer: raw_input_buffer.as_slice(),
                num_channels,
            }]),
            None,
            block_size,
            CurrentTime::from(0.0),
        )
        .unwrap();
    }

    #[test]
    fn it_should_not_panic_on_empty_inputs_and_outputs() {
        let mut id_gen = TestIdGenerator(0);
        let num_channels = 4;
        let mut node = PassthroughNode::new_with_channels(&mut id_gen, num_channels);
        let block_size = 256;

        run_process::<PassthroughNode, PortId>(
            &mut node,
            None,
            None,
            block_size,
            CurrentTime::from(0.0),
        )
        .unwrap();
    }

    #[ignore]
    #[test]
    fn it_should_handle_mismatched_inputs_and_outputs() {
        // is this helpful / necessary?
        todo!();
    }
}
