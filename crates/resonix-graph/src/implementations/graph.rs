use core::ops::Deref;

use crate::{
    ConnectionId, DescribePorts, GenerateId, GetNodeId, GetPortDescriptors, GraphError, Node,
    NodeId, ResonixConnection, ResonixGraph, ResonixId, ResonixNodeHandle, ResonixPortAddress,
    compare_nodes_by_priority,
};

use alloc::vec::Vec;
use hashbrown::{HashMap, HashSet};
use petgraph::{algo::tarjan_scc, graph as pgraph};

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

        let sccs = tarjan_scc(&self.graph);
        let sccs: Vec<Vec<ResonixId>> = sccs
            .into_iter()
            .map(|node_index_vec| {
                node_index_vec
                    .into_iter()
                    .map(|node_index| *self.petgraph_index_to_id_map.get(&node_index).unwrap())
                    .collect()
            })
            .collect();

        self.traverse_graph(&sccs, &mut visited_set, &mut |id, visited, _is_cyclical| {
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
    fn traverse_graph<F>(
        &self,
        sccs: &[Vec<ResonixId>],
        visited_set: &mut HashSet<ResonixId>,
        cb: &mut F,
    ) where
        F: FnMut(ResonixId, &mut HashSet<ResonixId>, bool),
    {
        let mut leaf_nodes: Vec<NodeId> = self
            .leaf_nodes
            .iter()
            .filter_map(|maybe_node_id| *maybe_node_id)
            .collect();

        leaf_nodes.sort_by(|id_a, id_b| {
            compare_nodes_by_priority(self.get_node(*id_a).unwrap(), self.get_node(*id_b).unwrap())
        });

        for leaf_node_id in leaf_nodes {
            // leaf nodes should not have already been visited
            debug_assert!(visited_set.get(&*leaf_node_id).is_none());

            if visited_set.get(&*leaf_node_id).is_some() {
                continue;
            }
            visited_set.insert(*leaf_node_id);
            self.visit_node(*leaf_node_id, sccs, visited_set, cb);
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

        // the cyclical node with the least-high priority id becomes a stand-in leaf-node
        cyclical_ids.sort_by(|id_a, id_b| {
            compare_nodes_by_priority(self.get_node(id_a).unwrap(), self.get_node(id_b).unwrap())
        });
        cyclical_ids.reverse();

        for cyclical_id in cyclical_ids {
            if visited_set.get(&cyclical_id).is_some() {
                continue;
            }
            visited_set.insert(cyclical_id);
            self.visit_node(cyclical_id, sccs, visited_set, cb);
        }
    }

    fn get_node<I: Deref<Target = ResonixId>>(&self, id: I) -> Option<&Node> {
        let index: usize = **id;
        let graph_item = self.graph_items.get(index);

        if let Some(Some(GraphItem::Node(node))) = graph_item {
            return Some(node);
        }

        None
    }

    fn visit_node<F>(
        &self,
        current_id: ResonixId,
        sccs: &[Vec<ResonixId>],
        visited_set: &mut HashSet<ResonixId>,
        cb: &mut F,
    ) where
        F: FnMut(ResonixId, &mut HashSet<ResonixId>, bool),
    {
        let is_cyclical = self.is_cyclical_node(current_id, sccs);

        // post-order traversal: visit all neighbors first
        let petgraph_index = self.id_to_pegraph_index_map.get(&current_id).unwrap();
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

        neighbor_ids.sort_by(|id_a, id_b| {
            compare_nodes_by_priority(self.get_node(id_a).unwrap(), self.get_node(id_b).unwrap())
        });

        // ignore the current node we're visiting
        let mut neighbor_ids: Vec<ResonixId> = neighbor_ids
            .into_iter()
            .filter(|&node_id| node_id != current_id)
            .collect();

        neighbor_ids.sort();

        let cyclical_neighbor_ids: Vec<ResonixId> = neighbor_ids
            .iter()
            .filter(|neighbor_id| self.is_cyclical_node(**neighbor_id, sccs))
            .copied()
            .collect();

        let acyclical_neighbor_ids: Vec<ResonixId> = neighbor_ids
            .iter()
            .filter(|neighbor_id| !self.is_cyclical_node(**neighbor_id, sccs))
            .copied()
            .collect();

        // TODO: if we're already computing a cycle, process all cyclical conections
        // before moving on to other neighbors (process cycle as a single unit)
        if is_cyclical {
            for cyclical_neighbor_id in cyclical_neighbor_ids {
                if visited_set.get(&cyclical_neighbor_id).is_some() {
                    continue;
                }
                visited_set.insert(cyclical_neighbor_id);
                self.visit_node(cyclical_neighbor_id, sccs, visited_set, cb);
            }

            for acyclical_neighbor_id in acyclical_neighbor_ids {
                if visited_set.get(&acyclical_neighbor_id).is_some() {
                    continue;
                }
                visited_set.insert(acyclical_neighbor_id);
                self.visit_node(acyclical_neighbor_id, sccs, visited_set, cb);
            }
        } else {
            for neighbor_id in neighbor_ids {
                if visited_set.get(&neighbor_id).is_some() {
                    continue;
                }
                visited_set.insert(neighbor_id);
                self.visit_node(neighbor_id, sccs, visited_set, cb);
            }
        }

        // now visit the leaf node last
        cb(current_id, visited_set, is_cyclical);
    }

    /// a node is cyclical if the new node to visit is in a SCC of length > 1
    /// OR if it's directly connected to itself
    fn is_cyclical_node(&self, id: ResonixId, sccs: &[Vec<ResonixId>]) -> bool {
        let scc = sccs
            .iter()
            .find(|scc| scc.iter().any(|scc_id| *id == **scc_id))
            .unwrap();

        if scc.len() > 1 {
            return true;
        }

        let petgraph_index = self.id_to_pegraph_index_map.get(&id).unwrap();
        let neighbor_indexes: Vec<_> = self
            .graph
            .neighbors_directed(*petgraph_index, petgraph::Direction::Incoming)
            .collect();
        let neighbor_ids: Vec<ResonixId> = neighbor_indexes
            .into_iter()
            .map(|neighbor_petgraph_index| {
                *self
                    .petgraph_index_to_id_map
                    .get(&neighbor_petgraph_index)
                    .unwrap()
            })
            .collect();

        neighbor_ids.iter().any(|neighbor_id| *neighbor_id == id)
    }

    pub fn visit_order(&self) -> Option<&[ResonixId]> {
        self.visit_order.as_deref()
    }
}

// TODO: move implementation to a sub-component rather than the graph itself
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
        // TODO: check that the adding the node is valid before making it
        // - node id should not already be in the graph
        // - node id should not be weirdly higher than the rest

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
        // TODO: do incremental updates in the future?
        // May not be necessary, since it can be computed in O(n) time,
        // where n is the number of nodes
        self.visit_order = Some(self.compute_new_visit_order());

        Ok(node_handle)
    }

    fn connect(
        &mut self,
        start_port_address: ResonixPortAddress,
        end_port_address: ResonixPortAddress,
    ) -> Result<&mut Self, GraphError> {
        // TODO: check that the connection is valid before making it
        // - connection should not already exist
        // - valid node id, port id, and direction
        // - must be compatible data-types
        // - must be the correct number of connections for both nodes
        // - must be correct node relationship node->node, param->node, etc.

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

            // ┌──────────┐ ┌──────────┐ ┌──────────┐  ┌──────────┐
            // │ Constant │ │ Multiply │ │ Constant │  │ Multiply │
            // └──────────┘ └──────────┘ └──────────┘  └──────────┘
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
                implementations::{ConstantNode, Graph, MultiplyNode, OutputNode},
            };

            use super::assert_visit_order_matches_handles;

            // ┌────────────────────┐
            // │ Constant Node id=0 │        (None)
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
            fn many_starter_nodes_one_leaf() {
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

            //                     ┌──────────────┐
            //                     │ Constant n=0 │
            //                     └──────────────┘
            //        ┌───────────────┬─────────┬───────────────┐
            // ┌──────▼──────┐ ┌──────▼──────┐  │               │
            // │ Output id=1 │ │ Output id=2 │  │               │
            // └─────────────┘ └─────────────┘  │               │
            //                           ┌──────▼──────┐ ┌──────▼──────┐
            //                           │ Output id=3 │ │ Output id=4 │
            //                           └─────────────┘ └─────────────┘
            #[test]
            fn one_starter_node_many_leaves() {
                let mut graph = Graph::new();

                let node_0 = ConstantNode::new_with_value(&mut graph, 0);
                let node_1 = OutputNode::new(&mut graph);
                let node_2 = OutputNode::new(&mut graph);
                let node_3 = OutputNode::new(&mut graph);
                let node_4 = OutputNode::new(&mut graph);

                let node_0 = graph.add(node_0).unwrap();
                let node_1 = graph.add(node_1).unwrap();
                let node_2 = graph.add(node_2).unwrap();
                let node_3 = graph.add(node_3).unwrap();
                let node_4 = graph.add(node_4).unwrap();

                graph
                    .connect(node_0.output_port_address(), node_1.input_port_address())
                    .unwrap()
                    .connect(node_0.output_port_address(), node_2.input_port_address())
                    .unwrap()
                    .connect(node_0.output_port_address(), node_3.input_port_address())
                    .unwrap()
                    .connect(node_0.output_port_address(), node_4.input_port_address())
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

            // ┌────────────────────┐
            // │ Constant Node id=0 │
            // └─────────┬──────────┘
            //           │┌───────────┐
            // ┌─────────▼▼─────────┐ │
            // │ Constant Node id=1 │ │
            // └─────────┬──────────┘ │
            //           └────────────┘
            #[test]
            fn loop_back_plus_parent_node() {
                let mut graph = Graph::new();

                let constant_node_0 = ConstantNode::new_with_value(&mut graph, 0);
                let constant_node_1 = ConstantNode::new_with_value(&mut graph, 1);

                let constant_node_0 = graph.add(constant_node_0).unwrap();
                let constant_node_1 = graph.add(constant_node_1).unwrap();

                graph
                    .connect(
                        constant_node_0.output_port_address(),
                        constant_node_1.set_constant_value_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        constant_node_1.output_port_address(),
                        constant_node_1.set_constant_value_port_address(),
                    )
                    .unwrap();

                let node_run_order = graph.visit_order();

                assert_visit_order_matches_handles(
                    &node_run_order,
                    &[Box::new(constant_node_0), Box::new(constant_node_1)],
                );
            }

            //          ┌────────────┐
            //┌─────────▼──────────┐ │
            //│ Constant Node id=0 │ │
            //└─────────┬┬─────────┘ │
            //          │└───────────┘
            //          │┌───────────┐
            //┌─────────▼▼─────────┐ │
            //│ Constant Node id=1 │ │
            //└─────────┬──────────┘ │
            //          └────────────┘
            #[test]
            fn two_connected_single_loop_back_nodes() {
                let mut graph = Graph::new();

                let constant_node_0 = ConstantNode::new_with_value(&mut graph, 0);
                let constant_node_1 = ConstantNode::new_with_value(&mut graph, 1);

                let constant_node_0 = graph.add(constant_node_0).unwrap();
                let constant_node_1 = graph.add(constant_node_1).unwrap();

                graph
                    .connect(
                        constant_node_0.output_port_address(),
                        constant_node_0.set_constant_value_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        constant_node_0.output_port_address(),
                        constant_node_1.set_constant_value_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        constant_node_1.output_port_address(),
                        constant_node_1.set_constant_value_port_address(),
                    )
                    .unwrap();

                let node_run_order = graph.visit_order();

                assert_visit_order_matches_handles(
                    &node_run_order,
                    &[Box::new(constant_node_0), Box::new(constant_node_1)],
                );
            }

            //         ┌────────────────┐
            //         │ ┌────────────┐ │
            // ┌───────▼─▼──────────┐ │ │
            // │ Constant Node id=0 │ │ │
            // └─────────┬┬─────────┘ │ │
            //           │└───────────┘ │
            //           │┌───────────┐ │
            // ┌─────────▼▼─────────┐ │ │
            // │ Constant Node id=1 │ │ │
            // └───────┬─┬──────────┘ │ │
            //         │ └────────────┘ │
            //         └────────────────┘
            #[test]
            fn two_nodes_every_connection() {
                let mut graph = Graph::new();

                let constant_node_0 = ConstantNode::new_with_value(&mut graph, 0);
                let constant_node_1 = ConstantNode::new_with_value(&mut graph, 1);

                let constant_node_0 = graph.add(constant_node_0).unwrap();
                let constant_node_1 = graph.add(constant_node_1).unwrap();

                graph
                    .connect(
                        constant_node_0.output_port_address(),
                        constant_node_0.set_constant_value_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        constant_node_0.output_port_address(),
                        constant_node_1.set_constant_value_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        constant_node_1.output_port_address(),
                        constant_node_1.set_constant_value_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        constant_node_1.output_port_address(),
                        constant_node_0.set_constant_value_port_address(),
                    )
                    .unwrap();

                let node_run_order = graph.visit_order();

                assert_visit_order_matches_handles(
                    &node_run_order,
                    &[Box::new(constant_node_0), Box::new(constant_node_1)],
                );
            }
        }

        mod mix_ayclic_and_cyclic {
            use std::boxed::Box;

            use crate::{
                ResonixGraph,
                implementations::{ConstantNode, Graph, MultiplyNode, OutputNode},
            };

            use super::assert_visit_order_matches_handles;

            //     ┌──────────────┐ ┌──────────────┐
            //     │ Constant n=0 │ │ Constant n=1 │
            //     └──────────────┘ └──────┬───────┘
            //                       ┌─────▼──────┐
            //                       │ Output n=2 │
            //                       └────────────┘
            // ┌─────────┐                ┌─────────┐
            // │ ┌───────▼──────┐ ┌───────▼───────┐ │
            // │ │ Constant n=3 │ │ Constant n=4  │ │
            // │ └───────┬──────┘ └────────┬─┬────┘ │
            // │         └──┐        ┌─────┘ └──────┘
            // │         ┌──▼────────▼──┐
            // │         │ Multiply n=5 │   ┌────────────┐
            // │         └──────┬─┬─────┘   │ Output n=7 │
            // └────────────────┘ │         └────────────┘
            //             ┌──────▼─────┐
            //             │ Output n=6 │
            //             └────────────┘
            #[test]
            fn mix_of_everything() {
                let mut graph = Graph::new();

                let node_0 = ConstantNode::new(&mut graph);
                let node_1 = ConstantNode::new(&mut graph);
                let node_2 = OutputNode::new(&mut graph);
                let node_3 = ConstantNode::new(&mut graph);
                let node_4 = ConstantNode::new(&mut graph);
                let node_5 = MultiplyNode::new(&mut graph);
                let node_6 = OutputNode::new(&mut graph);
                let node_7 = OutputNode::new(&mut graph);

                let node_0 = graph.add(node_0).unwrap();
                let node_1 = graph.add(node_1).unwrap();
                let node_2 = graph.add(node_2).unwrap();
                let node_3 = graph.add(node_3).unwrap();
                let node_4 = graph.add(node_4).unwrap();
                let node_5 = graph.add(node_5).unwrap();
                let node_6 = graph.add(node_6).unwrap();
                let node_7 = graph.add(node_7).unwrap();

                graph
                    .connect(node_1.output_port_address(), node_2.input_port_address())
                    .unwrap()
                    .connect(
                        node_3.output_port_address(),
                        node_5.left_operand_input_address(),
                    )
                    .unwrap()
                    .connect(
                        node_4.output_port_address(),
                        node_5.right_operand_input_address(),
                    )
                    .unwrap()
                    .connect(
                        node_5.output_port_address(),
                        node_3.set_constant_value_port_address(),
                    )
                    .unwrap()
                    .connect(
                        node_4.output_port_address(),
                        node_4.set_constant_value_port_address(),
                    )
                    .unwrap()
                    .connect(node_5.output_port_address(), node_6.input_port_address())
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
                        Box::new(node_5),
                        Box::new(node_6),
                        Box::new(node_7),
                    ],
                );
            }
        }

        mod priority_changes {

            use std::boxed::Box;

            use crate::{
                ResonixGraph,
                implementations::{ConstantNode, Graph, MultiplyNode, OutputNode},
            };

            use super::assert_visit_order_matches_handles;

            //     ┌──────────────┐ ┌──────────────┐
            //     │ Constant n=3 │ │ Constant n=5 │
            //     └──────────────┘ └──────┬───────┘
            //                       ┌─────▼──────┐
            //                       │ Output n=4 │
            //                       └────────────┘
            // ┌─────────┐                ┌─────────┐
            // │ ┌───────▼──────┐ ┌───────▼───────┐ │
            // │ │ Constant n=2 │ │ Constant n=7  │ │
            // │ └───────┬──────┘ └────────┬─┬────┘ │
            // │         └──┐        ┌─────┘ └──────┘
            // │         ┌──▼────────▼──┐
            // │         │ Multiply n=6 │   ┌────────────┐
            // │         └──────┬─┬─────┘   │ Output n=1 │
            // └────────────────┘ │         └────────────┘
            //             ┌──────▼─────┐
            //             │ Output n=0 │
            //             └────────────┘
            #[test]
            fn out_of_order_creation_produces_acceptable_result() {
                let mut graph = Graph::new();

                // create them in a weird order
                let node_0 = OutputNode::new(&mut graph);
                let node_1 = OutputNode::new(&mut graph);
                let node_2 = ConstantNode::new(&mut graph);
                let node_3 = ConstantNode::new(&mut graph);
                let node_4 = OutputNode::new(&mut graph);
                let node_5 = ConstantNode::new(&mut graph);
                let node_6 = MultiplyNode::new(&mut graph);
                let node_7 = ConstantNode::new(&mut graph);

                // add them out of order
                let node_2 = graph.add(node_2).unwrap();
                let node_6 = graph.add(node_6).unwrap();
                let node_4 = graph.add(node_4).unwrap();
                let node_5 = graph.add(node_5).unwrap();
                let node_3 = graph.add(node_3).unwrap();
                let node_0 = graph.add(node_0).unwrap();
                let node_7 = graph.add(node_7).unwrap();
                let node_1 = graph.add(node_1).unwrap();

                // connect them in weird order
                graph
                    .connect(node_5.output_port_address(), node_4.input_port_address())
                    .unwrap();
                graph
                    .connect(
                        node_2.output_port_address(),
                        node_6.left_operand_input_address(),
                    )
                    .unwrap();
                graph
                    .connect(node_6.output_port_address(), node_0.input_port_address())
                    .unwrap();
                graph
                    .connect(
                        node_7.output_port_address(),
                        node_6.right_operand_input_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        node_7.output_port_address(),
                        node_7.set_constant_value_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        node_6.output_port_address(),
                        node_2.set_constant_value_port_address(),
                    )
                    .unwrap();
                let node_run_order = graph.visit_order();

                assert_visit_order_matches_handles(
                    &node_run_order,
                    &[
                        Box::new(node_2),
                        Box::new(node_7),
                        Box::new(node_6),
                        Box::new(node_0),
                        Box::new(node_1),
                        Box::new(node_3),
                        Box::new(node_5),
                        Box::new(node_4),
                    ],
                );
            }
        }
    }
}
