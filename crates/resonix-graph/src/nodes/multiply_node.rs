use core::any::Any;

use alloc::{borrow::ToOwned, vec::Vec};

use crate::{
    GenerateId, ResonixAudioNode, ResonixData, ResonixDataList, ResonixDataResult, ResonixId,
    ResonixPortAddress, ResonixPortAddressDirection,
};

pub struct MultiplyNode {
    node_id: ResonixId,
    multiply_value: ResonixData,
    inputs: ResonixDataResult,
}

impl MultiplyNode {
    pub const MULTIPLY_PORT_ID: ResonixId = ResonixId::new(0usize);
    pub const OUTPUT_PORT_ID: ResonixId = ResonixId::new(1usize);

    pub fn new<G: GenerateId, D: Into<ResonixData>>(
        id_generator: &mut G,
        multiply_value: D,
    ) -> Self {
        let node_id = id_generator.generate_id();
        Self {
            node_id,
            multiply_value: multiply_value.into(),
            inputs: ResonixDataResult::default(),
        }
    }

    pub fn multiply_port_address(&self) -> ResonixPortAddress {
        ResonixPortAddress::new(
            self.node_id,
            Self::MULTIPLY_PORT_ID,
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
    fn next(&mut self) -> ResonixDataResult {
        let new_data = self.inputs.to_owned();
        let resonix_data_list_vec: Vec<ResonixDataList> = new_data
            .into_inner()
            .into_iter()
            .map(|connection_data| {
                let resonix_data_vec: Vec<ResonixData> = connection_data
                    .into_inner()
                    .into_iter()
                    .map(|data| match data {
                        ResonixData::F32(original_value_f32) => match self.multiply_value {
                            ResonixData::F32(multiplier_f32) => {
                                ResonixData::F32(original_value_f32 * multiplier_f32)
                            }
                            ResonixData::I32(multiplier_i32) => {
                                ResonixData::F32(original_value_f32 * multiplier_i32 as f32)
                            }
                        },
                        ResonixData::I32(original_value_i32) => match self.multiply_value {
                            ResonixData::F32(multiplier_f32) => {
                                ResonixData::I32(original_value_i32 * multiplier_f32 as i32)
                            }
                            ResonixData::I32(multiplier_i32) => {
                                ResonixData::I32(original_value_i32 * multiplier_i32)
                            }
                        },
                    })
                    .collect();
                let new_resonix_data_list: ResonixDataList = resonix_data_vec.into();
                new_resonix_data_list
            })
            .collect();
        resonix_data_list_vec.into()
    }

    fn assign_inputs(&mut self, inputs: ResonixDataResult) {
        self.inputs = inputs;
    }

    fn input_port_addresses(&self) -> Vec<ResonixPortAddress> {
        vec![self.multiply_port_address()]
    }

    fn output_port_addresses(&self) -> Vec<ResonixPortAddress> {
        vec![self.output_port_address()]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
