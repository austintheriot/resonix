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

pub struct ConstantNode {
    node_id: NodeId,
    constant_value: Option<Sample>,
    port_descriptors: ConstantNodePortDescriptors,
}

impl ConstantNode {
    pub fn new<G: GenerateId>(id_generator: &mut G) -> Audio<Self> {
        let node_id = NodeId::from(id_generator.generate_id());
        let port_descriptors = ConstantNodePortDescriptors::new(node_id);
        let constant_node = Self {
            node_id,
            constant_value: None,
            port_descriptors,
        };
        Audio(constant_node)
    }

    pub fn new_with_value<G: GenerateId, D: Into<Sample>>(
        id_generator: &mut G,
        constant_value: D,
    ) -> Audio<Self> {
        let node_id = NodeId::from(id_generator.generate_id());
        let port_descriptors = ConstantNodePortDescriptors::new(node_id);
        let constant_node = Self {
            node_id,
            constant_value: Some(constant_value.into()),
            port_descriptors,
        };
        Audio(constant_node)
    }
}

impl GetPortDescriptors<ConstantNodePortDescriptors> for ConstantNode {
    fn get_port_descriptors(&self) -> ConstantNodePortDescriptors {
        self.port_descriptors
    }
}

impl GetNodeId for ConstantNode {
    fn node_id(&self) -> Id {
        *self.node_id
    }
}

// TODO: implement true priority configurations
// for now, just use id for priority
impl GetPriority for ConstantNode {
    fn get_priority(&self) -> Priority {
        (**self.node_id).into()
    }
}

impl AudioNode for ConstantNode {
    fn process(
        &mut self,
        _inputs: &[Option<&[Sample]>],
        outputs: &mut [Option<&mut [Sample]>],
        _block_size: BlockSize,
    ) -> Result<(), AudioNodeRunError> {
        let output_port_slot = **ConstantNodePortDescriptors::OUTPUT_PORT_ID;
        let value = self.constant_value.unwrap_or_default();

        let Some(output_buffer) = outputs[output_port_slot].as_deref_mut() else {
            return Ok(());
        };

        for sample in output_buffer.iter_mut() {
            *sample = value;
        }

        Ok(())
    }
}

impl Deref for ConstantNode {
    type Target = ConstantNodePortDescriptors;

    fn deref(&self) -> &Self::Target {
        &self.port_descriptors
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::*;
    use crate::{
        primitives::{BlockSize, Sample},
        test_utils::TestIdGenerator,
    };

    /// Runs `node.process()` with no inputs and returns the output buffer contents.
    fn process_constant(node: &mut ConstantNode, block_size: usize) -> Vec<Sample> {
        let mut buf = vec![Sample::default(); block_size];
        {
            let mut outputs: Vec<Option<&mut [Sample]>> = vec![Some(buf.as_mut_slice())];
            node.process(&[], outputs.as_mut_slice(), BlockSize::new(block_size))
                .expect("process should not fail");
        }
        buf
    }

    #[test]
    fn outputs_zero_when_no_value_set() {
        let mut id_gen = TestIdGenerator(0);
        let mut node = ConstantNode::new(&mut id_gen).into_inner();
        let result = process_constant(&mut node, 1);
        assert_eq!(result, vec![Sample::from(0.0f32)]);
    }

    #[test]
    fn outputs_constant_value_for_single_sample() {
        let mut id_gen = TestIdGenerator(0);
        let mut node = ConstantNode::new_with_value(&mut id_gen, 5.0f32).into_inner();
        let result = process_constant(&mut node, 1);
        assert_eq!(result, vec![Sample::from(5.0f32)]);
    }

    #[test]
    fn fills_entire_block_with_constant_value() {
        let mut id_gen = TestIdGenerator(0);
        let mut node = ConstantNode::new_with_value(&mut id_gen, 3.0f32).into_inner();
        let result = process_constant(&mut node, 4);
        assert_eq!(
            result,
            vec![
                Sample::from(3.0f32),
                Sample::from(3.0f32),
                Sample::from(3.0f32),
                Sample::from(3.0f32),
            ]
        );
    }

    #[test]
    fn does_nothing_when_output_slot_is_not_connected() {
        let mut id_gen = TestIdGenerator(0);
        let mut node = ConstantNode::new_with_value(&mut id_gen, 5.0f32).into_inner();
        let mut outputs: Vec<Option<&mut [Sample]>> = vec![None];
        let result = node.process(&[], outputs.as_mut_slice(), BlockSize::new(1));
        assert!(result.is_ok());
    }
}

#[derive(Copy, Clone)]
pub struct ConstantNodePortDescriptors {
    node_id: NodeId,
    input_port_addresses: [PortAddress; 1],
    output_port_addresses: [PortAddress; 1],
}

impl ConstantNodePortDescriptors {
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            input_port_addresses: [Self::gen_set_constant_value_port_address(node_id)],
            output_port_addresses: [Self::gen_output_port_address(node_id)],
        }
    }
}

impl DescribePorts for ConstantNodePortDescriptors {
    fn input_port_addresses(&self) -> Option<&[PortAddress]> {
        Some(&self.input_port_addresses)
    }

    fn output_port_addresses(&self) -> Option<&[PortAddress]> {
        Some(&self.output_port_addresses)
    }
}

impl ConstantNodePortDescriptors {
    pub const SET_CONSTANT_VALUE_PORT_ID: PortId = PortId::new(0usize);
    pub const OUTPUT_PORT_ID: PortId = PortId::new(0usize);

    fn gen_set_constant_value_port_address(node_id: NodeId) -> PortAddress {
        PortAddress::new(
            node_id,
            Self::SET_CONSTANT_VALUE_PORT_ID,
            PortAddressDirection::Input,
        )
    }

    pub fn set_constant_value_port_address(&self) -> PortAddress {
        Self::gen_set_constant_value_port_address(self.node_id)
    }

    fn gen_output_port_address(node_id: NodeId) -> PortAddress {
        PortAddress::new(node_id, Self::OUTPUT_PORT_ID, PortAddressDirection::Output)
    }

    pub fn output_port_address(&self) -> PortAddress {
        Self::gen_output_port_address(self.node_id)
    }
}
