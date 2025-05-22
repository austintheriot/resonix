use core::ops::Deref;

use alloc::vec::Vec;

use crate::{
    DescribePorts, GenerateId, GetNodeId, GetPortDescriptors, ResonixAudioNode, ResonixData,
    ResonixDataList, ResonixId, ResonixPortAddress, ResonixPortAddressDirection,
};

pub struct ConstantNode {
    node_id: ResonixId,
    constant_value: ResonixData,
    port_descriptors: ConstantNodePortDescriptors,
}

impl ConstantNode {
    pub fn new<G: GenerateId>(id_generator: &mut G) -> Self {
        Self::new_with_value(id_generator, ResonixData::None)
    }

    pub fn new_with_value<G: GenerateId, D: Into<ResonixData>>(
        id_generator: &mut G,
        constant_value: D,
    ) -> Self {
        let node_id = id_generator.generate_id();
        let constant_value = constant_value.into();
        let port_descriptors = ConstantNodePortDescriptors::new(node_id);
        Self {
            node_id,
            constant_value,
            port_descriptors,
        }
    }
}

impl GetPortDescriptors<ConstantNodePortDescriptors> for ConstantNode {
    fn get_port_descriptors(&self) -> ConstantNodePortDescriptors {
        self.port_descriptors
    }
}

impl GetNodeId for ConstantNode {
    fn node_id(&self) -> ResonixId {
        self.node_id
    }
}

impl ResonixAudioNode for ConstantNode {
    fn next(&mut self) -> ResonixDataList {
        ResonixDataList::from([self.constant_value.clone()])
    }

    fn assign_inputs(&mut self, _inputs: ResonixDataList) {
        // assign any inputs to the constant value it holds
        todo!()
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
    node_id: ResonixId,
}

impl ConstantNodePortDescriptors {
    pub fn new(node_id: ResonixId) -> Self {
        Self { node_id }
    }
}

impl DescribePorts for ConstantNodePortDescriptors {
    fn input_port_addresses(&self) -> Vec<ResonixPortAddress> {
        vec![]
    }

    fn output_port_addresses(&self) -> Vec<ResonixPortAddress> {
        vec![self.output_port_address()]
    }
}

impl ConstantNodePortDescriptors {
    pub const OUTPUT_PORT_ID: ResonixId = ResonixId::new(0usize);

    pub fn output_port_address(&self) -> ResonixPortAddress {
        ResonixPortAddress::new(
            self.node_id,
            Self::OUTPUT_PORT_ID,
            ResonixPortAddressDirection::Output,
        )
    }
}
