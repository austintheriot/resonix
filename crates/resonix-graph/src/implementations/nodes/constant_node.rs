use core::ops::Deref;

use alloc::vec::Vec;

use crate::{
    primitives::{Data, DataList, Id, NodeId, PortAddress, PortAddressDirection, PortId, Priority},
    traits::{
        Audio, AudioNode, DescribePorts, GenerateId, GetNodeId, GetPortDescriptors, GetPriority,
    },
};

pub struct ConstantNode {
    node_id: NodeId,
    constant_value: Data,
    port_descriptors: ConstantNodePortDescriptors,
}

impl ConstantNode {
    pub fn new<G: GenerateId>(id_generator: &mut G) -> Audio<Self> {
        Self::new_with_value(id_generator, Data::None)
    }

    pub fn new_with_value<G: GenerateId, D: Into<Data>>(
        id_generator: &mut G,
        constant_value: D,
    ) -> Audio<Self> {
        let node_id = NodeId::from(id_generator.generate_id());
        let constant_value = constant_value.into();
        let port_descriptors = ConstantNodePortDescriptors::new(node_id);
        let constant_node = Self {
            node_id,
            constant_value,
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
    fn next(&mut self) -> DataList {
        DataList::from([self.constant_value.clone()])
    }

    fn assign_inputs(&mut self, mut inputs: DataList) {
        // TODO: return error value if more inputs given than expected
        self.constant_value = inputs.remove(0);
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
}

impl ConstantNodePortDescriptors {
    pub fn new(node_id: NodeId) -> Self {
        Self { node_id }
    }
}

impl DescribePorts for ConstantNodePortDescriptors {
    fn input_port_addresses(&self) -> Vec<PortAddress> {
        vec![self.set_constant_value_port_address()]
    }

    fn output_port_addresses(&self) -> Vec<PortAddress> {
        vec![self.output_port_address()]
    }
}

impl ConstantNodePortDescriptors {
    pub const SET_CONSTANT_VALUE_PORT_ID: PortId = PortId::new(0usize);
    pub const OUTPUT_PORT_ID: PortId = PortId::new(1usize);

    pub fn set_constant_value_port_address(&self) -> PortAddress {
        PortAddress::new(
            self.node_id,
            Self::SET_CONSTANT_VALUE_PORT_ID,
            PortAddressDirection::Input,
        )
    }

    pub fn output_port_address(&self) -> PortAddress {
        PortAddress::new(
            self.node_id,
            Self::OUTPUT_PORT_ID,
            PortAddressDirection::Output,
        )
    }
}
