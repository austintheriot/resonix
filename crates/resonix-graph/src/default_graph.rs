use crate::{
    Connectable, GenerateId, ResonixConnection, ResonixDataResult, ResonixGraph, ResonixId,
    ResonixNodeHandle, ResonixPortAddress,
};

use alloc::vec::Vec;
use hashbrown::HashMap;
use petgraph::graph as pgraph;

pub struct Graph {
    current_node_id: usize,
    connectables: Vec<Option<Connectable>>,
    visit_order: Option<Vec<ResonixId>>,
    port_data_map: HashMap<ResonixPortAddress, ResonixDataResult>,
    node_id_to_index_map: HashMap<ResonixId, pgraph::NodeIndex<pgraph::DefaultIx>>,
    index_to_node_id_map: HashMap<pgraph::NodeIndex<pgraph::DefaultIx>, ResonixId>,
    graph: petgraph::Graph<ResonixId, ResonixConnection>,
    // we want to preserve insertion order
    starter_nodes: Vec<ResonixId>,
}

impl Graph {
    fn new() -> Self {
        Graph {
            current_node_id: 0,
            connectables: Vec::new(),
            visit_order: None,
            port_data_map: HashMap::new(),
            node_id_to_index_map: HashMap::new(),
            graph: petgraph::Graph::<ResonixId, ResonixConnection>::new(),
            starter_nodes: Vec::new(),
            index_to_node_id_map: HashMap::new(),
        }
    }

    fn push_connectable(
        &mut self,
        node_id: ResonixId,
        connectable: Connectable,
        connectable_index: usize,
    ) {
        if self.connectables.len() <= connectable_index {
            self.connectables
                .resize_with(connectable_index + 1, Default::default);
        }
        self.connectables[*node_id] = Some(connectable);
    }

    fn calculate_new_visit_order(&self) -> Vec<ResonixId> {
        let mut visit_order: Vec<ResonixId> = Vec::new();

        for input_node_id in self.starter_nodes.iter() {
            let starting_node_index = self.node_id_to_index_map.get(input_node_id).unwrap();
            let mut dfs = petgraph::visit::Dfs::new(&self.graph, *starting_node_index);
            while let Some(node_index) = dfs.next(&self.graph) {
                let node_id = self.index_to_node_id_map.get(&node_index).unwrap();
                visit_order.push(*node_id);
            }
        }

        visit_order
    }

    pub fn visit_order(&self) -> Option<&[ResonixId]> {
        self.visit_order.as_deref()
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
        let connectable = connectable.into();
        let node_id = connectable.node_id();
        let node_handle = ResonixNodeHandle::new(node_id);

        // bookkeeping
        let index = self.graph.add_node(node_id);
        self.node_id_to_index_map.insert(node_id, index);
        self.index_to_node_id_map.insert(index, node_id);
        self.push_connectable(node_id, connectable, *node_id);

        // until a node as an incoming connection, it is a starter node
        self.starter_nodes.push(node_id);

        // must be recomputed on every modification
        self.visit_order = Some(self.calculate_new_visit_order());

        node_handle
    }

    fn connect(
        &mut self,
        start_port_address: ResonixPortAddress,
        end_port_address: ResonixPortAddress,
    ) -> Result<(), ()> {
        // TODO: check that the connection is valid before making it

        let start_node_id = start_port_address.node_id();
        let end_node_id = end_port_address.node_id();

        let start_index = self.node_id_to_index_map.get(&start_node_id).unwrap();
        let end_index = self.node_id_to_index_map.get(&end_node_id).unwrap();

        self.graph.add_edge(
            *start_index,
            *end_index,
            ResonixConnection::new(start_port_address, end_port_address),
        );

        // if it has a connection coming in now, it is no longer a starter node
        let end_node_ved_index = self
            .starter_nodes
            .iter()
            .find(|node_id| **node_id == end_node_id);
        if let Some(index) = end_node_ved_index {
            self.starter_nodes.remove(**index);
        }

        // must be recomputed on every modification
        self.visit_order = Some(self.calculate_new_visit_order());

        Ok(())
    }
}

#[cfg(test)]
mod graph_tests {
    mod initialization {
        use crate::Graph;

        #[test]
        fn it_should_allow_constructing_without_panicking() {
            Graph::new();
        }
    }

    mod node_visit_order {
        use std::vec::Vec;

        use crate::{ResonixId, ResonixNodeHandle};

        fn assert_visit_order_matches_handles(
            visit_order: &Option<&[ResonixId]>,
            node_handles: &[ResonixNodeHandle],
        ) {
            let node_handles_as_node_ids: Vec<ResonixId> = node_handles
                .iter()
                .map(|node_handle| node_handle.node_id())
                .collect();
            assert_eq!(visit_order.unwrap(), node_handles_as_node_ids.as_slice())
        }

        mod unconnected_graphs {
            use crate::{
                Audio, ConstantNode, Graph, MultiplyNode, ResonixGraph,
                default_graph::graph_tests::node_visit_order::assert_visit_order_matches_handles,
            };

            #[test]
            fn run_order_for_unconnected_nodes_should_be_their_insertion_order() {
                let mut graph = Graph::new();

                let constant_node_1 = ConstantNode::new(&mut graph);
                let multiply_node_1 = MultiplyNode::new(&mut graph, 2.0);
                let constant_node_2 = ConstantNode::new(&mut graph);
                let multiply_node_2 = MultiplyNode::new(&mut graph, 4.0);

                let constant_node_handle_1 = graph.add(Audio(constant_node_1));
                let multiply_node_handle_1 = graph.add(Audio(multiply_node_1));
                let constant_node_handle_2 = graph.add(Audio(constant_node_2));
                let multiply_node_handle_2 = graph.add(Audio(multiply_node_2));

                let visit_order = graph.visit_order();

                assert_visit_order_matches_handles(
                    &visit_order,
                    &[
                        constant_node_handle_1,
                        multiply_node_handle_1,
                        constant_node_handle_2,
                        multiply_node_handle_2,
                    ],
                );
            }
        }

        mod acyclic_graphs {
            use crate::{Audio, ConstantNode, Graph, MultiplyNode, ResonixGraph};

            use super::assert_visit_order_matches_handles;
            #[test]
            fn constant_node_to_multiply_node() {
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

                let node_run_order = graph.visit_order();

                assert_visit_order_matches_handles(
                    &node_run_order,
                    &[constant_node_handle, multiply_node_handle],
                );
            }
        }

        mod cyclic_graph {
            // TODO: implement cyclic graph tests
        }
    }
}
