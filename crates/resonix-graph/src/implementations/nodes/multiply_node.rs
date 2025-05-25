use core::ops::Deref;

use alloc::vec::Vec;

use crate::{
    Audio, DescribePorts, GenerateId, GetNodeId, GetPortDescriptors, GetPriority, NodeId, PortId,
    ResonixAudioNode, ResonixData, ResonixDataList, ResonixId, ResonixPortAddress,
    ResonixPortAddressDirection,
};

pub struct MultiplyNode {
    node_id: NodeId,
    left_operand_value: ResonixData,
    right_operand_value: ResonixData,
    port_descriptors: MultiplyNodePortDescriptors,
}

impl MultiplyNode {
    pub fn new<G: GenerateId>(id_generator: &mut G) -> Audio<Self> {
        Self::new_with_values(id_generator, ResonixData::None, ResonixData::None)
    }

    pub fn new_with_values<G: GenerateId, L: Into<ResonixData>, R: Into<ResonixData>>(
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
    fn node_id(&self) -> ResonixId {
        *self.node_id
    }
}

// TODO: implement true priority configurations
// for now, just use id for priority
impl GetPriority for MultiplyNode {
    fn get_priority(&self) -> crate::Priority {
        (**self.node_id).into()
    }
}

impl ResonixAudioNode for MultiplyNode {
    fn next(&mut self) -> ResonixDataList {
        let resonid_data = match self.left_operand_value {
            ResonixData::F32(original_value_f32) => match self.right_operand_value {
                ResonixData::F32(multiplier_f32) => {
                    ResonixData::F32(original_value_f32 * multiplier_f32)
                }
                ResonixData::I32(multiplier_i32) => {
                    ResonixData::F32(original_value_f32 * multiplier_i32 as f32)
                }
                ResonixData::None => ResonixData::Error,
                ResonixData::Error => ResonixData::Error,
            },
            ResonixData::I32(original_value_i32) => match self.right_operand_value {
                ResonixData::F32(multiplier_f32) => {
                    ResonixData::I32(original_value_i32 * multiplier_f32 as i32)
                }
                ResonixData::I32(multiplier_i32) => {
                    ResonixData::I32(original_value_i32 * multiplier_i32)
                }
                ResonixData::None => ResonixData::Error,
                ResonixData::Error => ResonixData::Error,
            },
            ResonixData::None => ResonixData::Error,
            ResonixData::Error => ResonixData::Error,
        };

        ResonixDataList::from(vec![resonid_data])
    }

    fn assign_inputs(&mut self, inputs: ResonixDataList) {
        inputs
            .into_inner()
            .into_iter()
            .enumerate()
            .for_each(|(i, data)| {
                let left_operand_index: usize = **self.left_operand_input_address().port_id();
                let right_operand_index: usize = **self.right_operand_input_address().port_id();

                if i == left_operand_index {
                    self.left_operand_value = data
                } else if i == right_operand_index {
                    self.right_operand_value = data
                }
            });
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
}

impl MultiplyNodePortDescriptors {
    pub const LEFT_OPERAND_INPUT_PORT_ID: PortId = PortId::new(0usize);
    pub const RIGHT_OPERAND_INPUT_PORT_ID: PortId = PortId::new(0usize);
    pub const OUTPUT_PORT_ID: PortId = PortId::new(1usize);

    pub fn new(node_id: NodeId) -> Self {
        Self { node_id }
    }

    pub fn left_operand_input_address(&self) -> ResonixPortAddress {
        ResonixPortAddress::new(
            self.node_id,
            Self::LEFT_OPERAND_INPUT_PORT_ID,
            ResonixPortAddressDirection::Input,
        )
    }

    pub fn right_operand_input_address(&self) -> ResonixPortAddress {
        ResonixPortAddress::new(
            self.node_id,
            Self::RIGHT_OPERAND_INPUT_PORT_ID,
            ResonixPortAddressDirection::Input,
        )
    }

    pub fn output_port_address(&self) -> ResonixPortAddress {
        ResonixPortAddress::new(
            self.node_id,
            Self::OUTPUT_PORT_ID,
            ResonixPortAddressDirection::Output,
        )
    }
}

impl DescribePorts for MultiplyNodePortDescriptors {
    fn input_port_addresses(&self) -> Vec<ResonixPortAddress> {
        vec![self.left_operand_input_address()]
    }

    fn output_port_addresses(&self) -> Vec<ResonixPortAddress> {
        vec![self.output_port_address()]
    }
}
