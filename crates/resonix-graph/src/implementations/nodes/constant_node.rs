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
        outputs: &mut OutputBuffers<'_>,
        _block_size: BlockSize,
    ) -> Result<(), AudioNodeRunError> {
        let Some(out) = outputs.get_mut(ConstantNodePortDescriptors::OUTPUT_PORT_ID) else {
            return Ok(());
        };

        let value = self.constant_value.unwrap_or_default();
        for sample in out.iter_mut() {
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
    pub const OUTPUT_PORT_ID: PortId = PortId::new(1usize);

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
