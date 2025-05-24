use core::ops::Deref;

use crate::{
    ConnectionId, DescribePorts, GenerateId, GetNodeId, GetPortDescriptors, GraphError, Node,
    NodeId, ResonixConnection, ResonixGraph, ResonixId, ResonixNodeHandle, ResonixPortAddress,
};

use alloc::vec::Vec;
use hashbrown::{HashMap, HashSet};
use petgraph::graph as pgraph;

enum GraphItem {
    Node(Node),

    // TODO: add/remove this type once we know we need it
    #[allow(dead_code)]
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
    leaf_nodes: Vec<Option<NodeId>>,
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
            leaf_nodes: Vec::new(),
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

    // If we track the leaf nodes, and then iterate UP through the tree,
    // rather than DOWN, and we do a POST-order traversal, where
    // all starting nodes/dependencies are guaranteed to be visited before
    // any leaf node that depends on them, that should guarantee no leaf
    // node is ever without its depedencies.
    fn compute_new_visit_order(&self) -> Vec<ResonixId> {
        let mut visit_order: Vec<ResonixId> = Vec::new();
        let mut visited_set: HashSet<ResonixId> = HashSet::new();

        self.traverse_graph(&mut visited_set, &mut |id, visited, _is_cyclical| {
            // TODO: maybe compute sccs and use them here?
            visited.insert(id);
            visit_order.push(id);
        });

        visit_order
    }

    // Every Node is assumed to be a starting node when it's inserted into the Graph.
    // As soon as it receives an incoming Connection, it is no longer a starting Node.
    // If that Connection forms a cycle, then that Node can become unreachable.
    //
    // For this reason, we begin by iterating through all starting Nodes, which,
    // by definition, are not cyclical.
    //
    // Then we iterate through all non-visited Nodes. Any non-visited Nodes
    // at this stage are, by definition, cyclical because they were not visited
    // by a starting Node.
    fn traverse_graph<F>(&self, visited_set: &mut HashSet<ResonixId>, cb: &mut F)
    where
        F: FnMut(ResonixId, &mut HashSet<ResonixId>, bool),
    {
        let mut leaf_nodes: Vec<NodeId> = self
            .leaf_nodes
            .iter()
            .filter_map(|maybe_node_id| *maybe_node_id)
            .collect();
        leaf_nodes.sort();

        for leaf_node_id in leaf_nodes {
            // leaf nodes should not have already been visited
            debug_assert!(visited_set.get(&*leaf_node_id).is_none());

            let is_cyclical = false;

            if visited_set.get(&*leaf_node_id).is_some() {
                continue;
            }
            visited_set.insert(*leaf_node_id);
            self.visit(*leaf_node_id, visited_set, cb, is_cyclical);
        }

        let mut cyclical_ids: Vec<ResonixId> = self
            .graph_items
            .iter()
            .filter_map(|graph_item| {
                // ignore all Connections
                if let Some(GraphItem::Node(node)) = graph_item {
                    let node_id = node.node_id();

                    // ignore all nodes already visited
                    if visited_set.get(&node_id).is_some() {
                        return None;
                    }

                    return Some(node_id);
                }

                None
            })
            .collect();

        // the cyclical node with the least-high priority id becomes
        // a stand-in leaf-node
        cyclical_ids.sort();
        cyclical_ids.reverse();

        for cyclical_id in cyclical_ids {
            let is_cyclical = true;
            if visited_set.get(&cyclical_id).is_some() {
                continue;
            }
            visited_set.insert(cyclical_id);
            self.visit(cyclical_id, visited_set, cb, is_cyclical);
        }
    }

    fn visit<F>(
        &self,
        id: ResonixId,
        visited_set: &mut HashSet<ResonixId>,
        cb: &mut F,
        is_cyclical: bool,
    ) where
        F: FnMut(ResonixId, &mut HashSet<ResonixId>, bool),
    {
        // post-order traversal: visit all neighbors first
        let petgraph_index = self.id_to_pegraph_index_map.get(&id).unwrap();
        let neighbor_indexes: Vec<_> = self
            .graph
            .neighbors_directed(*petgraph_index, petgraph::Direction::Incoming)
            .collect();
        let mut neighbor_ids: Vec<ResonixId> = neighbor_indexes
            .into_iter()
            .map(|neighbor_petgraph_index| {
                *self
                    .petgraph_index_to_id_map
                    .get(&neighbor_petgraph_index)
                    .unwrap()
            })
            .collect();

        // sort parent nodes by id--smaller gets higher priority
        // TODO: sort by explicity priority later?
        neighbor_ids.sort();

        // ignore the current node we're visiting
        let neighbor_ids: Vec<ResonixId> = neighbor_ids
            .into_iter()
            .filter(|&node_id| node_id != id)
            .collect();

        for neighbor_id in neighbor_ids {
            if visited_set.get(&neighbor_id).is_some() {
                continue;
            }
            self.visit(neighbor_id, visited_set, cb, is_cyclical);
        }

        // now visit the leaf node last
        cb(id, visited_set, is_cyclical);
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
        // until a node as an outgoing connection, it is a leaf node
        Graph::set_vec_map_item(*node_id, node_id, &mut self.leaf_nodes);

        // must be recomputed on every modification
        self.visit_order = Some(self.compute_new_visit_order());

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

        // if it has a connection going out now, it is no longer a leaf node
        let start_node_vec_index: usize = **start_node_id;
        core::mem::take(&mut self.leaf_nodes[start_node_vec_index]);

        // must be recomputed on every modification
        self.visit_order = Some(self.compute_new_visit_order());

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

        #[track_caller]
        fn assert_visit_order_matches_handles(
            visit_order: &Option<&[ResonixId]>,
            node_handles: &[Box<dyn GetNodeId>],
        ) {
            let node_handles_as_node_ids: Vec<ResonixId> = node_handles
                .iter()
                .map(|node_handle| node_handle.node_id())
                .collect();
            assert_eq!(
                visit_order.unwrap(),
                node_handles_as_node_ids.as_slice(),
                "Visit order is not equal"
            )
        }

        mod unconnected_graphs {
            use alloc::boxed::Box;

            use crate::{
                ResonixGraph,
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
                let multiply_node_handle_1 = graph.add(multiply_node_1).unwrap();
                let multiply_node_handle_2 = graph.add(multiply_node_2).unwrap();
                let constant_node_handle_1 = graph.add(constant_node_1).unwrap();
                let constant_node_handle_2 = graph.add(constant_node_2).unwrap();

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
                ResonixGraph,
                implementations::{ConstantNode, Graph, MultiplyNode},
            };

            use super::assert_visit_order_matches_handles;

            // ┌────────────────────┐
            // │ Constant Node id=0 │         None
            // └──────────────┬─────┘          │
            //                │                │
            //              ┌─▼────────────────▼─┐
            //              │ Multiply Node id=3 │
            //              └────────────────────┘
            #[test]
            fn single_connection() {
                let mut graph = Graph::new();

                let constant_node = ConstantNode::new(&mut graph);
                let multiply_node = MultiplyNode::new(&mut graph);

                let constant_node = graph.add(constant_node).unwrap();
                let multiply_node = graph.add(multiply_node).unwrap();

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

            // ┌────────────────────┐  ┌────────────────────┐
            // │ Constant Node id=0 │  │ Constant Node id=1 │
            // └──────────────┬─────┘  └───────┬────────────┘
            //                │                │
            //              ┌─▼────────────────▼─┐  ┌────────────────────┐
            //              │ Multiply Node id=3 │  │ Constant Node id=2 │
            //              └─────────────┬──────┘  └───────┬────────────┘
            //                            │                 │
            //                           ┌▼─────────────────▼─┐
            //                           │ Multiply Node id=3 │
            //                           └────────────────────┘
            #[test]
            fn simple_tree() {
                let mut graph = Graph::new();

                let node_0 = ConstantNode::new_with_value(&mut graph, 0);
                let node_1 = ConstantNode::new_with_value(&mut graph, 1);
                let node_2 = MultiplyNode::new(&mut graph);
                let node_3 = ConstantNode::new_with_value(&mut graph, 3);
                let node_4 = MultiplyNode::new(&mut graph);

                let node_0 = graph.add(node_0).unwrap();
                let node_1 = graph.add(node_1).unwrap();
                let node_2 = graph.add(node_2).unwrap();
                let node_3 = graph.add(node_3).unwrap();
                let node_4 = graph.add(node_4).unwrap();

                graph
                    .connect(
                        node_0.output_port_address(),
                        node_2.left_operand_input_address(),
                    )
                    .unwrap()
                    .connect(
                        node_1.output_port_address(),
                        node_2.right_operand_input_address(),
                    )
                    .unwrap()
                    .connect(
                        node_2.output_port_address(),
                        node_4.left_operand_input_address(),
                    )
                    .unwrap()
                    .connect(
                        node_3.output_port_address(),
                        node_4.right_operand_input_address(),
                    )
                    .unwrap();

                let node_run_order = graph.visit_order();

                assert_visit_order_matches_handles(
                    &node_run_order,
                    &[
                        Box::new(node_0),
                        Box::new(node_1),
                        Box::new(node_2),
                        Box::new(node_3),
                        Box::new(node_4),
                    ],
                );
            }
        }

        mod cyclic_graph {
            use std::boxed::Box;

            use crate::{
                ResonixGraph,
                implementations::{ConstantNode, Graph},
            };

            use super::assert_visit_order_matches_handles;

            //   ┌─────┐
            //┌──▼───┐ │
            //│ Node │ │
            //└──┬───┘ │
            //   └─────┘
            #[test]
            fn single_node() {
                let mut graph = Graph::new();

                let constant_node_1 = ConstantNode::new_with_value(&mut graph, 2);
                let constant_node_1 = graph.add(constant_node_1).unwrap();

                graph
                    .connect(
                        constant_node_1.output_port_address(),
                        constant_node_1.set_constant_value_port_address(),
                    )
                    .unwrap();

                let node_run_order = graph.visit_order();

                assert_visit_order_matches_handles(&node_run_order, &[Box::new(constant_node_1)]);
            }

            //           ┌────────────┐
            // ┌─────────▼──────────┐ │
            // │ Constant Node id=0 │ │
            // └─────────┬──────────┘ │
            // ┌─────────▼──────────┐ │
            // │ Constant Node id=1 │ │
            // └─────────┬──────────┘ │
            //           └────────────┘
            #[test]
            fn two_node() {
                let mut graph = Graph::new();

                let constant_node_1 = ConstantNode::new_with_value(&mut graph, 2);
                let constant_node_2 = ConstantNode::new_with_value(&mut graph, 3);

                let constant_node_1 = graph.add(constant_node_1).unwrap();
                let constant_node_2 = graph.add(constant_node_2).unwrap();

                graph
                    .connect(
                        constant_node_1.output_port_address(),
                        constant_node_2.set_constant_value_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        constant_node_2.output_port_address(),
                        constant_node_1.set_constant_value_port_address(),
                    )
                    .unwrap();

                let node_run_order = graph.visit_order();

                assert_visit_order_matches_handles(
                    &node_run_order,
                    &[Box::new(constant_node_1), Box::new(constant_node_2)],
                );
            }

            //           ┌────────────┐
            // ┌─────────▼──────────┐ │
            // │ Constant Node id=0 │ │
            // └─────────┬──────────┘ │
            // ┌─────────▼──────────┐ │
            // │ Constant Node id=1 │ │
            // └─────────┬──────────┘ │
            // ┌─────────▼──────────┐ │
            // │ Constant Node id=2 │ │
            // └─────────┬──────────┘ │
            //           └────────────┘
            #[ignore]
            #[test]
            fn three_node() {
                let mut graph = Graph::new();

                let constant_node_0 = ConstantNode::new_with_value(&mut graph, 0);
                let constant_node_1 = ConstantNode::new_with_value(&mut graph, 1);
                let constant_node_2 = ConstantNode::new_with_value(&mut graph, 2);

                let constant_node_0 = graph.add(constant_node_0).unwrap();
                let constant_node_1 = graph.add(constant_node_1).unwrap();
                let constant_node_2 = graph.add(constant_node_2).unwrap();

                graph
                    .connect(
                        constant_node_0.output_port_address(),
                        constant_node_1.set_constant_value_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        constant_node_1.output_port_address(),
                        constant_node_0.set_constant_value_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        constant_node_1.output_port_address(),
                        constant_node_2.set_constant_value_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        constant_node_2.output_port_address(),
                        constant_node_0.set_constant_value_port_address(),
                    )
                    .unwrap();

                let node_run_order = graph.visit_order();

                assert_visit_order_matches_handles(
                    &node_run_order,
                    &[
                        Box::new(constant_node_0),
                        Box::new(constant_node_1),
                        Box::new(constant_node_2),
                    ],
                );
            }
        }

        mod mix_ayclic_and_cyclic {

            // TODO: implement tests
        }
    }
}
