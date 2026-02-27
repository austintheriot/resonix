use core::ops::Deref;

use crate::{
    errors::AudioNodeRunError,
    primitives::{
        BlockSize, Id, NodeId, OutputBuffers, PortAddress, PortAddressDirection, PortId, Priority,
        Sample,
    },
    traits::{
        Audio, AudioNode, DescribePorts, GenerateId, GetNodeId, GetPortDescriptors, GetPriority,
    },
};

pub struct MultiplyNode {
    node_id: NodeId,
    left_operand_value: Sample,
    right_operand_value: Sample,
    port_descriptors: MultiplyNodePortDescriptors,
}

impl MultiplyNode {
    pub fn new<G: GenerateId>(id_generator: &mut G) -> Audio<Self> {
        Self::new_with_values(id_generator, Sample::default(), Sample::default())
    }

    pub fn new_with_values<G: GenerateId, L: Into<Sample>, R: Into<Sample>>(
        id_generator: &mut G,
        left_operand: L,
        right_operand: R,
    ) -> Audio<Self> {
        let node_id = NodeId::from(id_generator.generate_id());
        let left_operand_value = left_operand.into();
        let right_operand_value = right_operand.into();
        let port_descriptors = MultiplyNodePortDescriptors::new(node_id);

        let multiply_node = Self {
            node_id,
            right_operand_value,
            left_operand_value,
            port_descriptors,
        };
        Audio(multiply_node)
    }
}

impl GetPortDescriptors<MultiplyNodePortDescriptors> for MultiplyNode {
    fn get_port_descriptors(&self) -> MultiplyNodePortDescriptors {
        self.port_descriptors
    }
}

impl GetNodeId for MultiplyNode {
    fn node_id(&self) -> Id {
        *self.node_id
    }
}

// TODO: implement true priority configurations
// for now, just use id for priority
impl GetPriority for MultiplyNode {
    fn get_priority(&self) -> Priority {
        (**self.node_id).into()
    }
}

impl AudioNode for MultiplyNode {
    fn process(
        &mut self,
        inputs: &[Option<&[Sample]>],
        outputs: &mut OutputBuffers<'_>,
        _block_size: BlockSize,
    ) -> Result<(), AudioNodeRunError> {
        let left_block = inputs
            .get(**MultiplyNodePortDescriptors::LEFT_OPERAND_INPUT_PORT_ID)
            .copied()
            .flatten()
            .unwrap_or(&[]);
        let right_block = inputs
            .get(**MultiplyNodePortDescriptors::RIGHT_OPERAND_INPUT_PORT_ID)
            .copied()
            .flatten()
            .unwrap_or(&[]);

        let Some(out) = outputs.get_mut(MultiplyNodePortDescriptors::OUTPUT_PORT_ID) else {
            return Ok(());
        };

        for (i, sample) in out.iter_mut().enumerate() {
            let l = left_block
                .get(i)
                .copied()
                .unwrap_or(self.left_operand_value);
            let r = right_block
                .get(i)
                .copied()
                .unwrap_or(self.right_operand_value);
            *sample = Sample::new(*l * *r);
        }

        if let Some(last) = left_block.last() {
            self.left_operand_value = *last;
        }
        if let Some(last) = right_block.last() {
            self.right_operand_value = *last;
        }

        Ok(())
    }
}

impl Deref for MultiplyNode {
    type Target = MultiplyNodePortDescriptors;

    fn deref(&self) -> &Self::Target {
        &self.port_descriptors
    }
}

#[derive(Copy, Clone)]
pub struct MultiplyNodePortDescriptors {
    node_id: NodeId,
    input_port_addresses: [PortAddress; 2],
    output_port_addresses: [PortAddress; 1],
}

impl MultiplyNodePortDescriptors {
    pub const LEFT_OPERAND_INPUT_PORT_ID: PortId = PortId::new(0usize);
    pub const RIGHT_OPERAND_INPUT_PORT_ID: PortId = PortId::new(1usize);
    pub const OUTPUT_PORT_ID: PortId = PortId::new(2usize);

    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            input_port_addresses: [
                Self::gen_left_operand_input_address(node_id),
                Self::gen_right_operand_input_address(node_id),
            ],
            output_port_addresses: [Self::gen_output_port_address(node_id)],
        }
    }

    fn gen_left_operand_input_address(node_id: NodeId) -> PortAddress {
        PortAddress::new(
            node_id,
            Self::LEFT_OPERAND_INPUT_PORT_ID,
            PortAddressDirection::Input,
        )
    }

    pub fn left_operand_input_address(&self) -> PortAddress {
        Self::gen_left_operand_input_address(self.node_id)
    }

    fn gen_right_operand_input_address(node_id: NodeId) -> PortAddress {
        PortAddress::new(
            node_id,
            Self::RIGHT_OPERAND_INPUT_PORT_ID,
            PortAddressDirection::Input,
        )
    }

    pub fn right_operand_input_address(&self) -> PortAddress {
        Self::gen_right_operand_input_address(self.node_id)
    }

    fn gen_output_port_address(node_id: NodeId) -> PortAddress {
        PortAddress::new(node_id, Self::OUTPUT_PORT_ID, PortAddressDirection::Output)
    }

    pub fn output_port_address(&self) -> PortAddress {
        Self::gen_output_port_address(self.node_id)
    }
}

impl DescribePorts for MultiplyNodePortDescriptors {
    fn input_port_addresses(&self) -> Option<&[PortAddress]> {
        Some(&self.input_port_addresses)
    }

    fn output_port_addresses(&self) -> Option<&[PortAddress]> {
        Some(&self.output_port_addresses)
    }
}
