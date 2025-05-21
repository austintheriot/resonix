extern crate alloc;

mod graph {
    use std::collections::HashMap;

    use resonix_graph::{
        Connectable, GenerateId, ResonixDataResult, ResonixGraph, ResonixId, ResonixNodeHandle,
        ResonixPortAddress,
    };

    use petgraph::graph as pgraph;

    pub struct Graph {
        current_node_id: usize,
        connectables: Vec<Option<Connectable>>,
        node_run_order: Option<Vec<ResonixId>>,
        port_data_map: HashMap<ResonixPortAddress, ResonixDataResult>,
        port_address_to_index_map:
            HashMap<ResonixPortAddress, pgraph::NodeIndex<pgraph::DefaultIx>>,
        petgraph: petgraph::Graph<ResonixPortAddress, ()>,
    }

    impl Graph {
        fn new() -> Self {
            Graph {
                current_node_id: 0,
                connectables: Vec::new(),
                node_run_order: None,
                port_data_map: HashMap::new(),
                port_address_to_index_map: HashMap::new(),
                petgraph: petgraph::Graph::<ResonixPortAddress, ()>::new(),
            }
        }

        fn get<N: 'static, I: AsRef<ResonixId>>(&self, node_id: I) -> Option<&N> {
            // TODO: interesting, but probably not useful
            // can be deleted later if not needed.
            //
            // Demonstrates that it's possible to access Nodes of arbitrary type
            // from the outside and get temporary references to them
            let node_id = node_id.as_ref();
            let maybe_connectable = self.connectables.get(**node_id);
            match maybe_connectable.and_then(|inner| inner.as_ref()) {
                Some(Connectable::AudioNode(boxed)) => boxed.as_any().downcast_ref::<N>(),
                _ => None,
            }
        }

        fn calculate_node_run_order(&mut self) {
            todo!()
        }
    }

    impl GenerateId for Graph {
        fn generate_id(&mut self) -> ResonixId {
            let current_node_id = self.current_node_id;
            self.current_node_id += 1;
            current_node_id.into()
        }
    }

    impl ResonixGraph for Graph {
        fn add<C: Into<Connectable>>(&mut self, connectable: C) -> ResonixNodeHandle {
            let node_id = self.generate_id();
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

            let connectable_index = *node_id;

            // grow storage to match id
            if self.connectables.len() <= connectable_index {
                self.connectables
                    .resize_with(connectable_index + 1, Default::default);
            }
            self.connectables[*node_id] = Some(connectable);

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

    #[cfg(test)]
    mod graph_tests {
        use resonix_graph::{Audio, Param, ResonixGraph, ResonixNodeHandle};

        use crate::common::{
            graph::Graph,
            multiply_node,
            nodes::{ConstantNode, MultiplyNode},
        };

        #[test]
        fn it_should_allow_constructing_without_panicking() {
            Graph::new();
        }

        #[test]
        fn it_should_generate_correct_run_order() {
            let mut graph = Graph::new();

            let constant_node = ConstantNode::new(&mut graph);
            let multiply_node = MultiplyNode::new(&mut graph, 2.0);

            let constant_node_output_port_address = constant_node.output_port_address();
            let multiply_input_port_address = multiply_node.multiply_port_address();

            let constant_node_handle = graph.add(Audio(constant_node));
            let multiply_node_handle = graph.add(Audio(multiply_node));

            graph
                .connect(
                    constant_node_output_port_address,
                    multiply_input_port_address,
                )
                .unwrap();

            // TODO: test run order here
        }
    }
}

mod constant_node {
    use resonix_graph::{
        GenerateId, ResonixAudioNode, ResonixData, ResonixDataList, ResonixDataResult, ResonixId,
        ResonixPortAddress, ResonixPortAddressDirection,
    };

    pub struct ConstantNode {
        node_id: ResonixId,
    }

    impl ConstantNode {
        pub const OUTPUT_PORT_ID: ResonixId = ResonixId::new(0usize);

        pub fn new<G: GenerateId>(id_generator: &mut G) -> Self {
            let node_id = id_generator.generate_id();
            Self { node_id }
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

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
}

mod multiply_node {
    use resonix_graph::{
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

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
}

pub mod nodes {
    pub use super::constant_node::*;
    pub use super::multiply_node::*;
}
