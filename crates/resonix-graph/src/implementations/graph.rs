use core::ops::Deref;

use crate::{
    ConnectionId, DescribePorts, GenerateId, GetNodeId, GetPortDescriptors, GraphError, Node,
    NodeId, ResonixConnection, ResonixGraph, ResonixId, ResonixNodeHandle, ResonixPortAddress,
};

use alloc::vec::Vec;
use hashbrown::HashMap;
use petgraph::graph as pgraph;

enum GraphItem {
    Node(Node),
    Connection(ResonixConnection),
}

pub struct Graph {
    current_node_id: usize,
    // vec used as a HashMap for efficient lookups
    graph_items: Vec<Option<GraphItem>>,
    visit_order: Option<Vec<ResonixId>>,
    id_to_pegraph_index_map: HashMap<ResonixId, pgraph::NodeIndex<pgraph::DefaultIx>>,
    petgraph_index_to_id_map: HashMap<pgraph::NodeIndex<pgraph::DefaultIx>, ResonixId>,
    graph: petgraph::Graph<NodeId, ConnectionId>,
    starter_nodes: Vec<Option<NodeId>>,
    // will be necessary when processing data
    //port_data_map: HashMap<ResonixPortAddress, ResonixDataList>,
}

impl Graph {
    #[cfg(test)]
    fn new() -> Self {
        Graph {
            current_node_id: 0,
            graph_items: Vec::new(),
            visit_order: None,
            id_to_pegraph_index_map: HashMap::new(),
            graph: petgraph::Graph::<NodeId, ConnectionId>::new(),
            starter_nodes: Vec::new(),
            petgraph_index_to_id_map: HashMap::new(),
            //port_data_map: HashMap::new(),
        }
    }

    fn set_vec_map_item<K: Deref<Target = usize>, V>(
        key: K,
        value: V,
        vector: &mut Vec<Option<V>>,
    ) {
        let index: usize = *key.deref();
        if vector.len() <= index {
            vector.resize_with(index + 1, Default::default);
        }
        vector[index] = Some(value);
    }

    fn calculate_new_visit_order(&self) -> Vec<ResonixId> {
        let mut visit_order: Vec<ResonixId> = Vec::new();

        self.dfs(|node_id| {
            // TODO: implement Tarjan's algorith here for finding SCCs
            visit_order.push(node_id);
        });

        visit_order
    }

    fn dfs<F: FnMut(ResonixId)>(&self, mut cb: F) {
        // DFS, starting with id/creation order the starter nodes
        for input_node_id in self.starter_nodes.iter() {
            if let Some(input_node_id) = input_node_id {
                let starting_node_index =
                    self.id_to_pegraph_index_map.get(&**input_node_id).unwrap();

                let mut dfs = petgraph::visit::Dfs::new(&self.graph, *starting_node_index);
                while let Some(node_index) = dfs.next(&self.graph) {
                    let node_id = self.petgraph_index_to_id_map.get(&node_index).unwrap();
                    cb(*node_id);
                }
            }
        }
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
    fn add<P: DescribePorts, G: GetPortDescriptors<P>, C: Into<Node> + Deref<Target = G>>(
        &mut self,
        node: C,
    ) -> Result<ResonixNodeHandle<P>, GraphError> {
        let port_descriptors: P = node.get_port_descriptors();
        let node = node.into();
        let node_id = NodeId::from(node.node_id());
        let node_handle = ResonixNodeHandle::new(node_id, port_descriptors);

        // bookkeeping
        let index = self.graph.add_node(node_id);
        self.id_to_pegraph_index_map.insert(*node_id, index);
        self.petgraph_index_to_id_map.insert(index, *node_id);
        // petgraph only keeps ids--we keep the real values for easier bookkeeping
        Graph::set_vec_map_item(*node_id, GraphItem::Node(node), &mut self.graph_items);
        // until a node as an incoming connection, it is a starter node
        Graph::set_vec_map_item(*node_id, node_id, &mut self.starter_nodes);

        // must be recomputed on every modification
        self.visit_order = Some(self.calculate_new_visit_order());

        Ok(node_handle)
    }

    fn connect(
        &mut self,
        start_port_address: ResonixPortAddress,
        end_port_address: ResonixPortAddress,
    ) -> Result<&mut Self, GraphError> {
        // TODO: check that the connection is valid before making it

        let connection = ResonixConnection::new(self, start_port_address, end_port_address);
        let connection_id = connection.connection_id;

        let start_node_id = start_port_address.node_id();
        let start_index = *self.id_to_pegraph_index_map.get(&*start_node_id).unwrap();

        let end_node_id = end_port_address.node_id();
        let end_index = *self.id_to_pegraph_index_map.get(&*end_node_id).unwrap();

        Graph::set_vec_map_item(
            *connection_id,
            GraphItem::Connection(connection),
            &mut self.graph_items,
        );

        self.graph.add_edge(start_index, end_index, connection_id);

        // if it has a connection coming in now, it is no longer a starter node
        let end_node_vec_index: usize = **end_node_id;
        core::mem::take(&mut self.starter_nodes[end_node_vec_index]);

        // must be recomputed on every modification
        self.visit_order = Some(self.calculate_new_visit_order());

        Ok(self)
    }
}

#[cfg(test)]
mod graph_tests {
    mod initialization {
        use crate::implementations::Graph;

        #[test]
        fn it_should_allow_constructing_without_panicking() {
            Graph::new();
        }
    }

    mod node_visit_order {
        use alloc::{boxed::Box, vec::Vec};

        use crate::{GetNodeId, ResonixId};

        fn assert_visit_order_matches_handles(
            visit_order: &Option<&[ResonixId]>,
            node_handles: &[Box<dyn GetNodeId>],
        ) {
            let node_handles_as_node_ids: Vec<ResonixId> = node_handles
                .iter()
                .map(|node_handle| node_handle.node_id())
                .collect();
            assert_eq!(visit_order.unwrap(), node_handles_as_node_ids.as_slice())
        }

        mod unconnected_graphs {
            use alloc::boxed::Box;

            use crate::{
                Audio, ResonixGraph,
                implementations::{ConstantNode, Graph, MultiplyNode},
            };

            use super::assert_visit_order_matches_handles;

            // Constant Multiply Constant Multiply
            #[test]
            fn run_order_for_unconnected_nodes_should_be_their_creation_order() {
                let mut graph = Graph::new();

                let constant_node_1 = ConstantNode::new(&mut graph);
                let constant_node_2 = ConstantNode::new(&mut graph);
                let multiply_node_1 = MultiplyNode::new(&mut graph);
                let multiply_node_2 = MultiplyNode::new(&mut graph);

                // add in different order than creation
                let multiply_node_handle_1 = graph.add(Audio(multiply_node_1)).unwrap();
                let multiply_node_handle_2 = graph.add(Audio(multiply_node_2)).unwrap();
                let constant_node_handle_1 = graph.add(Audio(constant_node_1)).unwrap();
                let constant_node_handle_2 = graph.add(Audio(constant_node_2)).unwrap();

                let visit_order = graph.visit_order();

                assert_visit_order_matches_handles(
                    &visit_order,
                    &[
                        Box::new(constant_node_handle_1),
                        Box::new(constant_node_handle_2),
                        Box::new(multiply_node_handle_1),
                        Box::new(multiply_node_handle_2),
                    ],
                );
            }
        }

        mod acyclic_graphs {
            use alloc::boxed::Box;

            use crate::{
                Audio, ResonixGraph,
                implementations::{ConstantNode, Graph, MultiplyNode},
            };

            use super::assert_visit_order_matches_handles;

            // Constant None
            //    |      |
            //    L      R
            //    Multiply
            //       |
            //     Output
            #[test]
            fn constant_node_to_multiply_node() {
                let mut graph = Graph::new();

                let constant_node = ConstantNode::new(&mut graph);
                let multiply_node = MultiplyNode::new(&mut graph);

                let constant_node = graph.add(Audio(constant_node)).unwrap();
                let multiply_node = graph.add(Audio(multiply_node)).unwrap();

                graph
                    .connect(
                        constant_node.output_port_address(),
                        multiply_node.left_operand_input_address(),
                    )
                    .unwrap();

                let node_run_order = graph.visit_order();

                assert_visit_order_matches_handles(
                    &node_run_order,
                    &[Box::new(constant_node), Box::new(multiply_node)],
                );
            }

            // 2        3
            // |       /
            // L      R
            // Multiply  5
            // |        /
            // I      R
            // Multiply
            //    |
            // Output
            #[test]
            fn multiple_connections() {
                let mut graph = Graph::new();

                let constant_node_value_2 = ConstantNode::new_with_value(&mut graph, 2);
                let constant_node_value_3 = ConstantNode::new_with_value(&mut graph, 3);
                let multiply_node_1 = MultiplyNode::new(&mut graph);

                let constant_node_value_5 = ConstantNode::new_with_value(&mut graph, 5);
                let multiply_node_2 = MultiplyNode::new(&mut graph);

                let constant_node_value_2 = graph.add(Audio(constant_node_value_2)).unwrap();
                let constant_node_value_3 = graph.add(Audio(constant_node_value_3)).unwrap();
                let multiply_node_1 = graph.add(Audio(multiply_node_1)).unwrap();
                let constant_node_value_5 = graph.add(Audio(constant_node_value_5)).unwrap();
                let multiply_node_2 = graph.add(Audio(multiply_node_2)).unwrap();

                // TODO connect them
                graph
                    .connect(
                        constant_node_value_2.output_port_address(),
                        multiply_node_1.left_operand_input_address(),
                    )
                    .unwrap()
                    .connect(
                        constant_node_value_3.output_port_address(),
                        multiply_node_1.right_operand_input_address(),
                    )
                    .unwrap()
                    .connect(
                        multiply_node_1.output_port_address(),
                        multiply_node_2.left_operand_input_address(),
                    )
                    .unwrap()
                    .connect(
                        constant_node_value_5.output_port_address(),
                        multiply_node_2.right_operand_input_address(),
                    )
                    .unwrap();

                let node_run_order = graph.visit_order();

                graph.dfs(|id| {
                    println!("{:?}", id);
                });

                assert_visit_order_matches_handles(
                    &node_run_order,
                    &[
                        Box::new(constant_node_value_2),
                        Box::new(constant_node_value_3),
                        Box::new(constant_node_value_5),
                        Box::new(multiply_node_1),
                        Box::new(multiply_node_2),
                    ],
                );
            }
        }

        mod cyclic_graph {
            // TODO: implement cyclic graph tests
        }
    }
}
