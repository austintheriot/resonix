use core::ops::Deref;

use crate::{
    errors::AudioNodeRunError,
    primitives::{
        BlockSize, Id, NodeId, PortAddress, PortAddressDirection, PortDescriptor, PortId, Priority,
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
        implementations::{AudioBuffer, AudioBufferMut},
        primitives::{BlockSize, Sample},
        test_utils::TestIdGenerator,
        test_utils::{inputs_from_buffer_mapping, outputs_from_buffer_mapping},
    };

    fn run_process(
        node: &mut PassthroughNode,
        raw_input_buffer: Option<&[Sample]>,
        channels: usize,
        block_size: usize,
    ) -> Result<Vec<Sample>, AudioNodeRunError> {
        let inputs = if let Some(raw_input_buffer) = raw_input_buffer {
            inputs_from_buffer_mapping(&[(
                PassthroughPortDescriptors::INPUT_PORT_ID,
                raw_input_buffer,
                channels,
            )])
        } else {
            inputs_from_buffer_mapping::<PortId>(&[])
        };
        let inputs = inputs.as_slice();

        let mut raw_output_buffer = vec![Sample::default(); block_size * channels];
        let mut outputs = outputs_from_buffer_mapping(&mut [(
            PassthroughPortDescriptors::OUTPUT_PORT_ID,
            Some(raw_output_buffer.as_mut_slice()),
            channels,
        )]);
        let outputs = outputs.as_mut_slice();

        node.process(inputs, outputs, BlockSize::new(block_size))?;

        Ok(raw_output_buffer)
    }

    #[test]
    fn it_should_copy_single_channel_input_to_output() {
        let mut id_gen = TestIdGenerator(0);
        let mut node = PassthroughNode::new_with_channels(&mut id_gen, 1);

        let raw_input_buffer: Vec<Sample> =
            [0.0, 1.0, 2.0, 3.0].into_iter().map(Sample::from).collect();
        let channels = 1;
        let block_size = raw_input_buffer.len();

        let output = run_process(
            &mut node,
            Some(raw_input_buffer.as_slice()),
            channels,
            block_size,
        )
        .unwrap();

        assert_eq!(output, raw_input_buffer);
    }

    #[test]
    fn it_should_copy_multi_channel_input_to_output() {
        let mut id_gen = TestIdGenerator(0);
        let mut node = PassthroughNode::new_with_channels(&mut id_gen, 1);

        let raw_input_buffer: Vec<Sample> =
            [0.0, 1.0, 2.0, 3.0].into_iter().map(Sample::from).collect();
        let channels = 4;
        let block_size = raw_input_buffer.len() / channels;

        let output = run_process(
            &mut node,
            Some(raw_input_buffer.as_slice()),
            channels,
            block_size,
        )
        .unwrap();

        assert_eq!(output, raw_input_buffer);
    }

    #[test]
    fn it_should_not_panic_on_empty_inputs() {
        let mut id_gen = TestIdGenerator(0);
        let mut node = PassthroughNode::new_with_channels(&mut id_gen, 1);

        let raw_input_buffer: Vec<Sample> =
            [0.0, 1.0, 2.0, 3.0].into_iter().map(Sample::from).collect();
        let channels = 4;
        let block_size = raw_input_buffer.len() / channels;

        run_process(&mut node, None, channels, block_size).unwrap();
    }

    #[test]
    fn it_should_not_panic_on_empty_outputs() {
        let mut id_gen = TestIdGenerator(0);
        let mut node = PassthroughNode::new_with_channels(&mut id_gen, 1);

        node.process::<AudioBuffer<'_>, AudioBufferMut<'_>>(&[], &mut [], BlockSize::from(256))
            .unwrap();
    }

    #[ignore]
    #[test]
    fn it_should_handle_mismatched_inputs_and_outputs() {
        // is this helpful / necessary?
        todo!();
    }
}
