use core::ops::Deref;

use crate::{
    errors::AudioNodeRunError,
    primitives::{
        BlockSize, Id, NodeId, PortAddress, PortAddressDirection, PortId, Priority, Sample,
    },
    traits::{
        Audio, AudioNode, DescribePorts, GenerateId, GetNodeId, GetPortDescriptors, GetPriority,
    },
};

pub struct OutputNode {
    node_id: NodeId,
    input_value: Option<Sample>,
    port_descriptors: OutputNodePortDescriptors,
}

impl OutputNode {
    pub fn new<G: GenerateId>(id_generator: &mut G) -> Audio<Self> {
        let node_id: NodeId = id_generator.generate_id().into();
        let output_node = Self {
            node_id,
            input_value: None,
            port_descriptors: OutputNodePortDescriptors::new(node_id),
        };
        Audio(output_node)
    }
}

impl GetPortDescriptors<OutputNodePortDescriptors> for OutputNode {
    fn get_port_descriptors(&self) -> OutputNodePortDescriptors {
        self.port_descriptors
    }
}

impl GetNodeId for OutputNode {
    fn node_id(&self) -> Id {
        *self.node_id
    }
}

// TODO: implement true priority configurations
// for now, just use id for priority
impl GetPriority for OutputNode {
    fn get_priority(&self) -> Priority {
        (**self.node_id).into()
    }
}

impl AudioNode for OutputNode {
    fn process(
        &mut self,
        inputs: &[Option<&[Sample]>],
        outputs: &mut [Option<&mut [Sample]>],
        _block_size: BlockSize,
    ) -> Result<(), AudioNodeRunError> {
        let input_port_slot = **OutputNodePortDescriptors::INPUT_PORT_ID;
        let output_port_slot = **OutputNodePortDescriptors::EXTERNAL_OUTPUT_PORT_ID;

        let Some(output_block) = outputs[output_port_slot].as_deref_mut() else {
            return Ok(());
        };

        // TODO: handle None case (self-reference)
        let input_block = inputs[input_port_slot].unwrap_or(&[]);

        for (out, &inp) in output_block.iter_mut().zip(input_block.iter()) {
            *out = inp;
        }

        self.input_value = input_block.last().copied();

        Ok(())
    }
}

impl Deref for OutputNode {
    type Target = OutputNodePortDescriptors;

    fn deref(&self) -> &Self::Target {
        &self.port_descriptors
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::*;
    use crate::{primitives::{BlockSize, Sample}, test_utils::TestIdGenerator};

    /// Runs `node.process()` with the given input and returns the output buffer contents.
    fn process_output(node: &mut OutputNode, input: &[Sample], block_size: usize) -> Vec<Sample> {
        let mut out_buf = vec![Sample::default(); block_size];
        let inputs: Vec<Option<&[Sample]>> = vec![Some(input)];
        {
            let mut outputs: Vec<Option<&mut [Sample]>> = vec![Some(out_buf.as_mut_slice())];
            node.process(inputs.as_slice(), outputs.as_mut_slice(), BlockSize::new(block_size))
                .expect("process should not fail");
        }
        out_buf
    }

    #[test]
    fn copies_input_to_output() {
        let mut id_gen = TestIdGenerator(0);
        let mut node = OutputNode::new(&mut id_gen).into_inner();
        let input = [Sample::from(7.0f32)];
        let result = process_output(&mut node, &input, 1);
        assert_eq!(result, vec![Sample::from(7.0f32)]);
    }

    #[test]
    fn copies_multi_sample_block() {
        let mut id_gen = TestIdGenerator(0);
        let mut node = OutputNode::new(&mut id_gen).into_inner();
        let input = [
            Sample::from(1.0f32),
            Sample::from(2.0f32),
            Sample::from(3.0f32),
            Sample::from(4.0f32),
        ];
        let result = process_output(&mut node, &input, 4);
        assert_eq!(
            result,
            vec![
                Sample::from(1.0f32),
                Sample::from(2.0f32),
                Sample::from(3.0f32),
                Sample::from(4.0f32),
            ]
        );
    }

    #[test]
    fn does_nothing_when_external_output_slot_is_not_connected() {
        let mut id_gen = TestIdGenerator(0);
        let mut node = OutputNode::new(&mut id_gen).into_inner();
        let input = [Sample::from(5.0f32)];
        let inputs: Vec<Option<&[Sample]>> = vec![Some(input.as_slice())];
        let mut outputs: Vec<Option<&mut [Sample]>> = vec![None];
        let result = node.process(inputs.as_slice(), outputs.as_mut_slice(), BlockSize::new(1));
        assert!(result.is_ok());
    }

    #[test]
    fn partial_input_only_writes_available_samples() {
        let mut id_gen = TestIdGenerator(0);
        let mut node = OutputNode::new(&mut id_gen).into_inner();
        // input shorter than output — zip stops at input length, rest remains default (0.0)
        let input = [Sample::from(1.0f32), Sample::from(2.0f32)];
        let result = process_output(&mut node, &input, 4);
        assert_eq!(
            result,
            vec![
                Sample::from(1.0f32),
                Sample::from(2.0f32),
                Sample::from(0.0f32),
                Sample::from(0.0f32),
            ]
        );
    }
}

#[derive(Copy, Clone)]
pub struct OutputNodePortDescriptors {
    node_id: NodeId,
    input_port_addresses: [PortAddress; 1],
    external_port_address: [PortAddress; 1],
}

impl DescribePorts for OutputNodePortDescriptors {
    fn input_port_addresses(&self) -> Option<&[PortAddress]> {
        Some(&self.input_port_addresses)
    }

    fn external_output_port_addresses(&self) -> Option<&[PortAddress]> {
        Some(&self.external_port_address)
    }
}

impl OutputNodePortDescriptors {
    pub const INPUT_PORT_ID: PortId = PortId::new(0usize);
    pub const EXTERNAL_OUTPUT_PORT_ID: PortId = PortId::new(0usize);

    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            input_port_addresses: [Self::gen_input_port_address(node_id)],
            external_port_address: [Self::gen_external_output_port_address(node_id)],
        }
    }

    fn gen_input_port_address(node_id: NodeId) -> PortAddress {
        PortAddress::new(node_id, Self::INPUT_PORT_ID, PortAddressDirection::Input)
    }

    fn gen_external_output_port_address(node_id: NodeId) -> PortAddress {
        PortAddress::new(
            node_id,
            Self::EXTERNAL_OUTPUT_PORT_ID,
            PortAddressDirection::ExternalOutput,
        )
    }

    pub fn input_port_address(&self) -> PortAddress {
        Self::gen_input_port_address(self.node_id)
    }

    pub fn external_output_port_address(&self) -> PortAddress {
        Self::gen_external_output_port_address(self.node_id)
    }
}
