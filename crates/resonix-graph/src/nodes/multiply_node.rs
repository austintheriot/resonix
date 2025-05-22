use alloc::vec::Vec;

use crate::{
    GenerateId, HasPortDescriptors, ResonixAudioNode, ResonixData, ResonixDataList, ResonixId,
    ResonixPortAddress, ResonixPortAddressDirection,
};

pub struct MultiplyNode {
    node_id: ResonixId,
    left_operator_value: ResonixData,
    right_operator_value: ResonixData,
}

pub struct MultiplyNodePortDescriptors {
    node_id: ResonixId,
}

impl HasPortDescriptors<MultiplyNodePortDescriptors> for MultiplyNode {
    fn port_descriptors(&self) -> MultiplyNodePortDescriptors {
        MultiplyNodePortDescriptors {
            node_id: self.node_id,
        }
    }
}

impl MultiplyNode {
    pub const LEFT_OPERATOR_INPUT_PORT_ID: ResonixId = ResonixId::new(0usize);
    pub const RIGHT_OPERATOR_INPUT_PORT_ID: ResonixId = ResonixId::new(0usize);
    pub const OUTPUT_PORT_ID: ResonixId = ResonixId::new(1usize);

    pub fn new<G: GenerateId>(id_generator: &mut G) -> Self {
        let node_id = id_generator.generate_id();
        Self {
            node_id,
            right_operator_value: ResonixData::None,
            left_operator_value: ResonixData::None,
        }
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

    fn input_port_addresses(&self) -> Vec<ResonixPortAddress> {
        vec![self.left_operator_input_address()]
    }

    fn output_port_addresses(&self) -> Vec<ResonixPortAddress> {
        vec![self.output_port_address()]
    }

    fn node_id(&self) -> ResonixId {
        self.node_id
    }
}
