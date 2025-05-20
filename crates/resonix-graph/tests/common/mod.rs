extern crate alloc;

mod graph {
    use std::collections::HashMap;

    use resonix_graph::{
        Connectable, ResonixDataResult, ResonixGraph, ResonixId, ResonixNodeHandle,
        ResonixPortAddress,
    };

    use petgraph::graph as pgraph;

    pub struct Graph {
        current_node_id: usize,
        // probably not needed
        //connectables_map: HashMap<ResonixId, Connectable>,
        port_data_map: HashMap<ResonixPortAddress, ResonixDataResult>,
        port_address_to_index_map:
            HashMap<ResonixPortAddress, pgraph::NodeIndex<pgraph::DefaultIx>>,
        petgraph: petgraph::Graph<ResonixPortAddress, ()>,
    }

    impl Graph {
        fn get_and_increment_id(&mut self) -> ResonixId {
            let current_node_id = self.current_node_id;
            self.current_node_id += 1;
            current_node_id.into()
        }
    }

    impl ResonixGraph for Graph {
        fn add<C: Into<Connectable>>(&mut self, connectable: C) -> ResonixNodeHandle {
            let node_id = self.get_and_increment_id();
            let node_handle = ResonixNodeHandle::new(node_id);
            let connectable = connectable.into();
            let input_port_addresses = connectable.input_port_addresses();
            let output_port_addresses = connectable.output_port_addresses();

            input_port_addresses
                .into_iter()
                .chain(output_port_addresses.into_iter())
                .for_each(|port_address| {
                    let index = self.petgraph.add_node(port_address);
                    self.port_address_to_index_map.insert(port_address, index);
                });

            node_handle
        }

        fn connect(
            &mut self,
            port_a: ResonixPortAddress,
            port_b: ResonixPortAddress,
        ) -> Result<(), ()> {
            let index_a = self.port_address_to_index_map.get(&port_a).unwrap();
            let index_b = self.port_address_to_index_map.get(&port_b).unwrap();
            self.petgraph.add_edge(*index_a, *index_b, ());
            Ok(())
        }
    }
}

mod constant_node {
    use resonix_graph::{
        ResonixAudioNode, ResonixData, ResonixDataList, ResonixDataResult, ResonixId,
        ResonixPortAddress, ResonixPortAddressDirection,
    };

    pub struct ConstantNode {
        node_id: ResonixId,
    }

    impl ConstantNode {
        pub const OUTPUT_PORT_ID: ResonixId = ResonixId::new(0usize);

        pub fn new<I: Into<ResonixId>>(id: I) -> Self {
            Self { node_id: id.into() }
        }

        pub fn output_port_address(&self) -> ResonixPortAddress {
            ResonixPortAddress::new(
                self.node_id,
                Self::OUTPUT_PORT_ID,
                ResonixPortAddressDirection::Output,
            )
        }
    }

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

        fn input_port_addresses(&self) -> Vec<ResonixPortAddress> {
            vec![]
        }

        fn output_port_addresses(&self) -> Vec<ResonixPortAddress> {
            vec![self.output_port_address()]
        }
    }
}

mod multiply_node {
    use resonix_graph::{
        ResonixAudioNode, ResonixData, ResonixDataList, ResonixDataResult, ResonixId,
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

        pub fn new<I: Into<ResonixId>, D: Into<ResonixData>>(id: I, multiply_value: D) -> Self {
            Self {
                node_id: id.into(),
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
    }
}

pub mod nodes {
    pub use super::constant_node::*;
    pub use super::multiply_node::*;
}
