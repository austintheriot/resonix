extern crate alloc;

mod graph {
    use resonix_graph::Connectable;

    pub struct Graph {
        connectables: Vec<Connectable>,
    }
}

mod constant_node {
    use resonix_graph::{
        ResonixAudioNode, ResonixData, ResonixDataList, ResonixDataResult, ResonixId,
    };

    pub struct ConstantNode {
        id: ResonixId,
    }

    impl ConstantNode {
        pub fn new<I: Into<ResonixId>>(id: I) -> Self {
            Self { id: id.into() }
        }
    }

    impl ResonixAudioNode for ConstantNode {
        fn next(&mut self) -> ResonixDataResult {
            let connection_data: Vec<ResonixDataList> =
                vec![ResonixDataList::from([ResonixData::F32(1.0)])];

            connection_data.into()
        }

        fn node_id(&self) -> resonix_graph::ResonixId {
            self.id
        }

        fn assign_inputs(&mut self, _inputs: ResonixDataResult) {
            // it takes no inputs
            unimplemented!()
        }
    }
}

mod multiply_node {
    use resonix_graph::{
        ResonixAudioNode, ResonixData, ResonixDataList, ResonixDataResult, ResonixId,
        ResonixPortHandle,
    };

    pub struct MultiplyNode {
        id: ResonixId,
        multiply_value: ResonixData,
        inputs: ResonixDataResult,
    }

    impl MultiplyNode {
        pub const MULTIPLY_PORT_ID: ResonixId = ResonixId::new(0u32);
        pub const OUTPUT_PORT_ID: ResonixId = ResonixId::new(1u32);

        pub fn new<I: Into<ResonixId>, D: Into<ResonixData>>(id: I, multiply_value: D) -> Self {
            Self {
                id: id.into(),
                multiply_value: multiply_value.into(),
                inputs: ResonixDataResult::default(),
            }
        }

        pub fn multiply_port(&self) -> ResonixPortHandle {
            ResonixPortHandle::new(self.id, MultiplyNode::MULTIPLY_PORT_ID)
        }

        pub fn output_port(&self) -> ResonixPortHandle {
            ResonixPortHandle::new(self.id, MultiplyNode::OUTPUT_PORT_ID)
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

        fn node_id(&self) -> ResonixId {
            self.id
        }
    }
}

pub mod nodes {
    pub use super::constant_node::*;
    pub use super::multiply_node::*;
}
