extern crate alloc;

mod constant_node {
    use resonix_graph::{ResonixAudioNode, ResonixData, ResonixDataList, ResonixDataResult};

    pub struct ConstantNode;

    impl ResonixAudioNode for ConstantNode {
        fn next(&mut self) -> ResonixDataResult {
            let connection_data: Vec<ResonixDataList> =
                vec![ResonixDataList::from([ResonixData::F32(1.0)])];

            connection_data.into()
        }

        fn assign_inputs(&mut self, _inputs: ResonixDataResult) {
            // it takes no inputs
            unimplemented!()
        }
    }
}

mod multiply_node {
    use resonix_graph::{ResonixAudioNode, ResonixData, ResonixDataList, ResonixDataResult};

    pub struct MultiplyNode {
        multiply_value: ResonixData,
        inputs: ResonixDataResult,
    }

    impl MultiplyNode {
        pub fn new<D: Into<ResonixData>>(multiply_value: D) -> Self {
            Self {
                multiply_value: multiply_value.into(),
                inputs: ResonixDataResult::default(),
            }
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
    }
}

pub mod nodes {
    pub use super::constant_node::*;
    pub use super::multiply_node::*;
}
