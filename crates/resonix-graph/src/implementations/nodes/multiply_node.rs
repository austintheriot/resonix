use core::ops::Deref;

use alloc::vec::Vec;

use crate::{
    DescribePorts, GenerateId, GetNodeId, GetPortDescriptors, ResonixAudioNode, ResonixData,
    ResonixDataList, ResonixId, ResonixPortAddress, ResonixPortAddressDirection,
};

pub struct MultiplyNode {
    node_id: ResonixId,
    left_operator_value: ResonixData,
    right_operator_value: ResonixData,
    port_descriptors: MultiplyNodePortDescriptors,
}

impl MultiplyNode {
    pub fn new<G: GenerateId>(id_generator: &mut G) -> Self {
        Self::new_with_values(id_generator, ResonixData::None, ResonixData::None)
    }

    pub fn new_with_values<G: GenerateId, L: Into<ResonixData>, R: Into<ResonixData>>(
        id_generator: &mut G,
        left_operator: L,
        right_operator: R,
    ) -> Self {
        let node_id = id_generator.generate_id();
        let left_operator_value = left_operator.into();
        let right_operator_value = right_operator.into();
        let port_descriptors = MultiplyNodePortDescriptors::new(node_id);

        Self {
            node_id,
            right_operator_value,
            left_operator_value,
            port_descriptors,
        }
    }
}

impl GetPortDescriptors<MultiplyNodePortDescriptors> for MultiplyNode {
    fn get_port_descriptors(&self) -> MultiplyNodePortDescriptors {
        self.port_descriptors
    }
}

impl GetNodeId for MultiplyNode {
    fn node_id(&self) -> ResonixId {
        self.node_id
    }
}

impl ResonixAudioNode for MultiplyNode {
    fn next(&mut self) -> ResonixDataList {
        let resonid_data = match self.left_operator_value {
            ResonixData::F32(original_value_f32) => match self.right_operator_value {
                ResonixData::F32(multiplier_f32) => {
                    ResonixData::F32(original_value_f32 * multiplier_f32)
                }
                ResonixData::I32(multiplier_i32) => {
                    ResonixData::F32(original_value_f32 * multiplier_i32 as f32)
                }
                ResonixData::None => ResonixData::Error,
                ResonixData::Error => ResonixData::Error,
            },
            ResonixData::I32(original_value_i32) => match self.right_operator_value {
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
                let left_operator_index = *self.left_operator_input_address().port_id();
                let right_operator_index = *self.right_operator_input_address().port_id();

                if i == left_operator_index {
                    self.left_operator_value = data
                } else if i == right_operator_index {
                    self.right_operator_value = data
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
    node_id: ResonixId,
}

impl MultiplyNodePortDescriptors {
    pub const LEFT_OPERATOR_INPUT_PORT_ID: ResonixId = ResonixId::new(0usize);
    pub const RIGHT_OPERATOR_INPUT_PORT_ID: ResonixId = ResonixId::new(0usize);
    pub const OUTPUT_PORT_ID: ResonixId = ResonixId::new(1usize);

    pub fn new(node_id: ResonixId) -> Self {
        Self { node_id }
    }

    pub fn left_operator_input_address(&self) -> ResonixPortAddress {
        ResonixPortAddress::new(
            self.node_id,
            Self::LEFT_OPERATOR_INPUT_PORT_ID,
            ResonixPortAddressDirection::Input,
        )
    }

    pub fn right_operator_input_address(&self) -> ResonixPortAddress {
        ResonixPortAddress::new(
            self.node_id,
            Self::RIGHT_OPERATOR_INPUT_PORT_ID,
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
        vec![self.left_operator_input_address()]
    }

    fn output_port_addresses(&self) -> Vec<ResonixPortAddress> {
        vec![self.output_port_address()]
    }
}
