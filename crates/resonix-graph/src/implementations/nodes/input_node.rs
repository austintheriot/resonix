use core::ops::Deref;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    errors::AudioNodeRunError,
    primitives::{
        BlockSize, CurrentTime, Id, NodeId, PortAddress, PortAddressDirection, PortDescriptor,
        PortId, Priority,
    },
    traits::{
        AudioBuffer, AudioBufferMut, AudioNode, DescribePorts, GenerateId, GetNodeId,
        GetPortDescriptors, GetPriority,
    },
};

#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct InputNode {
    node_id: NodeId,
    port_descriptors: InputNodePortDescriptors,
}

impl InputNode {
    pub fn new<G: GenerateId>(id_generator: &mut G) -> Self {
        let default_num_channels = 1;

        Self::new_with_channels(id_generator, default_num_channels)
    }

    pub fn new_with_channels<G: GenerateId>(id_generator: &mut G, channels: usize) -> Self {
        let node_id: NodeId = id_generator.generate_id().into();

        Self {
            node_id,
            port_descriptors: InputNodePortDescriptors::new(node_id, channels),
        }
    }
}

impl GetPortDescriptors<InputNodePortDescriptors> for InputNode {
    fn get_port_descriptors(&self) -> InputNodePortDescriptors {
        self.port_descriptors
    }
}

impl GetNodeId for InputNode {
    fn node_id(&self) -> Id {
        *self.node_id
    }
}

// TODO: implement true priority configurations
// for now, just use id for priority
impl GetPriority for InputNode {
    fn get_priority(&self) -> Priority {
        (**self.node_id).into()
    }
}

impl AudioNode for InputNode {
    fn process<A: AudioBuffer, M: AudioBufferMut>(
        &mut self,
        inputs: &[Option<A>],
        outputs: &mut [Option<M>],
        _block_size: BlockSize,
        _current_time: CurrentTime,
    ) -> Result<(), AudioNodeRunError> {
        // no output buffer to write to, nothing to do
        let Some(output_buffer) = outputs
            .get_mut(**InputNodePortDescriptors::OUTPUT_PORT_ID)
            .and_then(|o| o.as_mut())
        else {
            return Ok(());
        };

        // no input data to read from, nothing to do
        let Some(external_input_buffer) = inputs
            .get(**InputNodePortDescriptors::EXTERNAL_INPUT_PORT_ID)
            .and_then(|i| i.as_ref())
        else {
            return Ok(());
        };

        // copy external output data to output
        for (input_block, output_block) in external_input_buffer
            .channels_iter()
            .unwrap()
            .zip(output_buffer.channels_iter_mut().unwrap())
        {
            for (input_sample, output_sample) in input_block.iter().zip(output_block.iter_mut()) {
                *output_sample = *input_sample;
            }
        }

        Ok(())
    }
}

impl Deref for InputNode {
    type Target = InputNodePortDescriptors;

    fn deref(&self) -> &Self::Target {
        &self.port_descriptors
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
    fn copies_single_channel_external_input_to_output() {
        let mut id_gen = TestIdGenerator(0);
        let num_channels = 1;
        let mut node = InputNode::new_with_channels(&mut id_gen, num_channels);

        let raw_external_input_buffer: Vec<Sample> =
            [0.0, 1.0, 2.0, 3.0].into_iter().map(Sample::from).collect();
        let mut raw_output_buffer: Vec<Sample> = [0.0; 4].into_iter().map(Sample::from).collect();
        let block_size = raw_external_input_buffer.len();

        run_process(
            &mut node,
            Some(&[InputBufferKeyMapping {
                id: InputNodePortDescriptors::EXTERNAL_INPUT_PORT_ID,
                buffer: raw_external_input_buffer.as_slice(),
                num_channels,
            }]),
            Some(&mut [OutputBufferKeyMapping {
                id: InputNodePortDescriptors::OUTPUT_PORT_ID,
                buffer: Some(raw_output_buffer.as_mut_slice()),
                num_channels,
            }]),
            block_size,
            CurrentTime::from(0.0),
        )
        .unwrap();

        assert_eq!(raw_external_input_buffer, raw_output_buffer);
    }

    #[test]
    fn copies_single_multi_channel_external_input_to_output() {
        let mut id_gen = TestIdGenerator(0);
        let num_channels = 4;
        let mut node = InputNode::new_with_channels(&mut id_gen, num_channels);

        let raw_external_input_buffer: Vec<Sample> =
            [0.0, 1.0, 2.0, 3.0].into_iter().map(Sample::from).collect();
        let mut raw_output_buffer: Vec<Sample> = [0.0; 4].into_iter().map(Sample::from).collect();
        let block_size = raw_external_input_buffer.len();

        run_process(
            &mut node,
            Some(&[InputBufferKeyMapping {
                id: InputNodePortDescriptors::EXTERNAL_INPUT_PORT_ID,
                buffer: raw_external_input_buffer.as_slice(),
                num_channels,
            }]),
            Some(&mut [OutputBufferKeyMapping {
                id: InputNodePortDescriptors::OUTPUT_PORT_ID,
                buffer: Some(raw_output_buffer.as_mut_slice()),
                num_channels,
            }]),
            block_size,
            CurrentTime::from(0.0),
        )
        .unwrap();

        assert_eq!(raw_external_input_buffer, raw_output_buffer);
    }

    #[test]
    fn doesnt_throw_when_external_input_slot_is_not_connected() {
        let mut id_gen = TestIdGenerator(0);
        let num_channels = 1;
        let mut node = InputNode::new_with_channels(&mut id_gen, num_channels);
        let mut raw_output_buffer: Vec<Sample> = [0.0; 4].into_iter().map(Sample::from).collect();
        let block_size = raw_output_buffer.len();

        run_process(
            &mut node,
            None,
            Some(&mut [OutputBufferKeyMapping {
                id: InputNodePortDescriptors::OUTPUT_PORT_ID,
                buffer: Some(raw_output_buffer.as_mut_slice()),
                num_channels,
            }]),
            block_size,
            CurrentTime::from(0.0),
        )
        .unwrap();
    }

    #[test]
    fn doesnt_throw_when_output_slot_is_not_connected() {
        let mut id_gen = TestIdGenerator(0);
        let num_channels = 1;
        let mut node = InputNode::new_with_channels(&mut id_gen, num_channels);

        let raw_external_input_buffer: Vec<Sample> =
            [0.0, 1.0, 2.0, 3.0].into_iter().map(Sample::from).collect();
        let block_size = raw_external_input_buffer.len();

        run_process(
            &mut node,
            Some(&[InputBufferKeyMapping {
                id: InputNodePortDescriptors::EXTERNAL_INPUT_PORT_ID,
                buffer: raw_external_input_buffer.as_slice(),
                num_channels,
            }]),
            None,
            block_size,
            CurrentTime::from(0.0),
        )
        .unwrap();
    }

    #[test]
    fn input_len_less_than_output_doesnt_write_too_much() {
        let mut id_gen = TestIdGenerator(0);
        let num_channels = 1;
        let mut node = InputNode::new_with_channels(&mut id_gen, num_channels);

        let raw_external_input_buffer: Vec<Sample> =
            [0.0, 1.0, 2.0, 3.0].into_iter().map(Sample::from).collect();
        let mut raw_output_buffer: Vec<Sample> = [0.0; 8].into_iter().map(Sample::from).collect();

        // input channel samples, followed by silence
        let expected_output_buffer: Vec<Sample> = raw_external_input_buffer
            .clone()
            .into_iter()
            .chain(raw_output_buffer.clone().into_iter().take(4))
            .collect();

        let block_size = raw_external_input_buffer.len();

        run_process(
            &mut node,
            Some(&[InputBufferKeyMapping {
                id: InputNodePortDescriptors::EXTERNAL_INPUT_PORT_ID,
                buffer: raw_external_input_buffer.as_slice(),
                num_channels,
            }]),
            Some(&mut [OutputBufferKeyMapping {
                id: InputNodePortDescriptors::OUTPUT_PORT_ID,
                buffer: Some(raw_output_buffer.as_mut_slice()),
                num_channels,
            }]),
            block_size,
            CurrentTime::from(0.0),
        )
        .unwrap();

        assert_eq!(raw_output_buffer, expected_output_buffer);
    }

    #[test]
    fn input_len_greater_than_output_doesnt_throw() {
        let mut id_gen = TestIdGenerator(0);
        let num_channels = 1;
        let mut node = InputNode::new_with_channels(&mut id_gen, num_channels);

        let raw_external_input_buffer: Vec<Sample> =
            [0.0, 1.0, 2.0, 3.0].into_iter().map(Sample::from).collect();
        let mut raw_output_buffer: Vec<Sample> = [0.0; 2].into_iter().map(Sample::from).collect();

        let expected_output_buffer: Vec<Sample> = raw_external_input_buffer
            .clone()
            .into_iter()
            .take(raw_output_buffer.len())
            .collect();

        let block_size = raw_external_input_buffer.len();

        run_process(
            &mut node,
            Some(&[InputBufferKeyMapping {
                id: InputNodePortDescriptors::EXTERNAL_INPUT_PORT_ID,
                buffer: raw_external_input_buffer.as_slice(),
                num_channels,
            }]),
            Some(&mut [OutputBufferKeyMapping {
                id: InputNodePortDescriptors::OUTPUT_PORT_ID,
                buffer: Some(raw_output_buffer.as_mut_slice()),
                num_channels,
            }]),
            block_size,
            CurrentTime::from(0.0),
        )
        .unwrap();

        assert_eq!(raw_output_buffer, expected_output_buffer);
    }
}

#[derive(Copy, Clone)]
pub struct InputNodePortDescriptors {
    node_id: NodeId,
    input_port_descriptors: [PortDescriptor; 1],
    external_port_descriptors: [PortDescriptor; 1],
}

impl DescribePorts for InputNodePortDescriptors {
    fn input_ports(&self) -> Option<&[PortDescriptor]> {
        Some(&self.input_port_descriptors)
    }

    fn output_ports(&self) -> Option<&[PortDescriptor]> {
        Some(&self.external_port_descriptors)
    }
}

impl InputNodePortDescriptors {
    pub const EXTERNAL_INPUT_PORT_ID: PortId = PortId::new(0usize);
    pub const OUTPUT_PORT_ID: PortId = PortId::new(0usize);

    pub fn new(node_id: NodeId, channels: usize) -> Self {
        Self {
            node_id,
            input_port_descriptors: [PortDescriptor {
                address: Self::gen_input_port_address(node_id),
                channels,
            }],
            external_port_descriptors: [PortDescriptor {
                address: Self::gen_external_output_port_address(node_id),
                channels,
            }],
        }
    }

    fn gen_input_port_address(node_id: NodeId) -> PortAddress {
        PortAddress::new(
            node_id,
            Self::EXTERNAL_INPUT_PORT_ID,
            PortAddressDirection::ExternalInput,
        )
    }

    fn gen_external_output_port_address(node_id: NodeId) -> PortAddress {
        PortAddress::new(node_id, Self::OUTPUT_PORT_ID, PortAddressDirection::Output)
    }

    pub fn input_port_address(&self) -> PortAddress {
        Self::gen_input_port_address(self.node_id)
    }

    pub fn external_output_port_address(&self) -> PortAddress {
        Self::gen_external_output_port_address(self.node_id)
    }
}
