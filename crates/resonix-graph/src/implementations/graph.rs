use core::ops::Deref;

use crate::{
    errors::{GraphAddError, GraphConnectionError, GraphRunError},
    primitives::{
        BlockSize, BufferPool, Connection, ConnectionId, Id, Node, NodeHandle, NodeId,
        NodeProcessContext, PortAddress, PortAddressDirection, Sample,
    },
    traits::{DescribePorts, GenerateId, GetNodeId, GetPortDescriptors},
    utils::{IntMap, IntSet, compare_nodes_by_priority},
};

use alloc::vec::Vec;
use hashbrown::{HashMap, HashSet};
use petgraph::algo::tarjan_scc;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use wasm_bindgen::prelude::wasm_bindgen;

/// Per-node storage of which `ConnectionId` backs each port slot.
/// Allocated once at `add()` time; updated during `connect()`.
/// A `None` entry means the port is unconnected.
struct NodePortMap {
    inputs: alloc::boxed::Box<[Option<ConnectionId>]>,
    outputs: alloc::boxed::Box<[Option<ConnectionId>]>,
}

enum GraphItem {
    Node(Node),

    // TODO: add/remove this type once we know we need it
    #[allow(dead_code)]
    Connection(Connection),
}

#[derive(Default)]
struct GraphIdGenerator {
    current_node_id: usize,
}

impl GenerateId for GraphIdGenerator {
    fn generate_id(&mut self) -> Id {
        let current_node_id = self.current_node_id;
        self.current_node_id += 1;
        current_node_id.into()
    }
}

#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
pub struct Graph {
    id_generator: GraphIdGenerator,
    graph_items: IntMap<Id, GraphItem>,
    /// Per-node connection-ID index. Populated in `add()`, updated in
    /// `connect()`. Used to build short-lived `NodeProcessContext`s in `run()`.
    node_port_maps: IntMap<NodeId, NodePortMap>,
    visit_order: Option<Vec<Id>>,
    id_to_pegraph_index_map: HashMap<Id, petgraph::graph::NodeIndex<petgraph::graph::DefaultIx>>,
    petgraph_index_to_id_map: HashMap<petgraph::graph::NodeIndex<petgraph::graph::DefaultIx>, Id>,
    port_address_to_connection_id_map: HashMap<PortAddress, ConnectionId>,
    #[allow(dead_code)]
    graph_run_connection_id_set: IntSet<ConnectionId>,
    graph: petgraph::Graph<NodeId, ConnectionId>,
    leaf_nodes: IntSet<NodeId>,
    block_size: BlockSize,
    buffer_pool: BufferPool,
}

impl Graph {
    pub fn with_block_size(block_size: impl Into<BlockSize>) -> Self {
        use crate::utils::IntMap;

        Graph {
            id_generator: GraphIdGenerator::default(),
            graph_items: IntMap::default(),
            node_port_maps: IntMap::default(),
            visit_order: None,
            id_to_pegraph_index_map: HashMap::new(),
            graph: petgraph::Graph::<NodeId, ConnectionId>::new(),
            leaf_nodes: IntSet::default(),
            petgraph_index_to_id_map: HashMap::new(),
            port_address_to_connection_id_map: HashMap::new(),
            block_size: block_size.into(),
            buffer_pool: BufferPool::default(),
            graph_run_connection_id_set: IntSet::default(),
        }
    }

    #[cfg(test)]
    fn new() -> Self {
        Self::with_block_size(1)
    }

    fn id_generator(&mut self) -> &mut impl GenerateId {
        &mut self.id_generator
    }

    // If we track the leaf nodes, and then iterate UP/backwards through the tree,
    // rather than DOWN, and we do a POST-order traversal, where
    // all starting nodes/dependencies are guaranteed to be visited before
    // any leaf node that depends on them, that should guarantee no leaf
    // node is ever run without its dependencies (in an acylic graph).
    fn compute_new_visit_order(&self) -> Vec<Id> {
        let mut visit_order: Vec<Id> = Vec::new();
        let mut visited_set: HashSet<Id> = HashSet::new();

        let sccs: Vec<Vec<Id>> = tarjan_scc(&self.graph)
            .into_iter()
            .map(|node_index_vec| {
                node_index_vec
                    .into_iter()
                    .map(|node_index| *self.petgraph_index_to_id_map.get(&node_index).unwrap())
                    .collect()
            })
            .collect();

        self.traverse_graph(&sccs, &mut visited_set, &mut |id| {
            visit_order.push(id);
        });

        visit_order
    }

    // Stricly speaking, this function does all the heavy lifting of figuring out the
    // graph order, so there's no necessity to pre-compute the graph order, but computing
    // this ahead-of-time significantly decreases number of runtime calculations that
    // are required for every audio frame otherwise.
    //
    // Every Node is assumed to be a leaf node when it's inserted into the Graph.
    // As soon as it receives an outgoing Connection, it is no longer a leaf Node.
    // If that Connection forms a cycle, then that Node can become unreachable.
    //
    // For this reason, we begin by iterating through all leaf Nodes, which,
    // by definition, are not cyclical.
    //
    // Then we iterate through all non-visited Nodes. Any non-visited Nodes
    // at this stage are, by definition, cyclical because they were not visited
    // from a a path that includes a leaf Node.
    fn traverse_graph<F>(&self, sccs: &[Vec<Id>], visited_set: &mut HashSet<Id>, cb: &mut F)
    where
        F: FnMut(Id),
    {
        let mut leaf_nodes: Vec<NodeId> = self.leaf_nodes.iter().copied().collect();

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

        let mut cyclical_ids: Vec<Id> = self
            .graph_items
            .iter()
            .filter_map(|(node_id, graph_item)| {
                // ignore all Connections
                if let GraphItem::Node(_node) = graph_item {
                    // ignore all nodes already visited
                    if visited_set.get(node_id).is_some() {
                        return None;
                    }

                    return Some(*node_id);
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

    fn get_node<I: Deref<Target = Id>>(&self, id: I) -> Option<&Node> {
        let graph_item = self.graph_items.get(id.deref());

        if let Some(GraphItem::Node(node)) = graph_item {
            return Some(node);
        }

        None
    }

    fn visit_node<F>(
        &self,
        current_id: Id,
        sccs: &[Vec<Id>],
        visited_set: &mut HashSet<Id>,
        cb: &mut F,
    ) where
        F: FnMut(Id),
    {
        let petgraph_index = self.id_to_pegraph_index_map.get(&current_id).unwrap();
        let mut neighbor_ids: Vec<Id> = self
            .graph
            // traverse backwards/UP the graph from the bottom/leaf nodes
            .neighbors_directed(*petgraph_index, petgraph::Direction::Incoming)
            //convert petgraph index to node id
            .map(|neighbor_petgraph_index| {
                *self
                    .petgraph_index_to_id_map
                    .get(&neighbor_petgraph_index)
                    .unwrap()
            })
            // ignore the current node we're visiting
            .filter(|&node_id| node_id != current_id)
            .collect();

        neighbor_ids.sort_by(|id_a, id_b| {
            compare_nodes_by_priority(self.get_node(id_a).unwrap(), self.get_node(id_b).unwrap())
        });

        let cyclical_neighbor_ids = neighbor_ids
            .iter()
            .filter(|neighbor_id| self.is_cyclical_node(**neighbor_id, sccs));

        let acyclical_neighbor_ids = neighbor_ids
            .iter()
            .filter(|neighbor_id| !self.is_cyclical_node(**neighbor_id, sccs));

        // cycles are treated with priority to isolate their weird run order
        // before moving onto acyclic parts of the graph
        for cyclical_neighbor_id in cyclical_neighbor_ids {
            if visited_set.get(cyclical_neighbor_id).is_some() {
                continue;
            }
            visited_set.insert(*cyclical_neighbor_id);
            self.visit_node(*cyclical_neighbor_id, sccs, visited_set, cb);
        }

        for acyclical_neighbor_id in acyclical_neighbor_ids {
            if visited_set.get(acyclical_neighbor_id).is_some() {
                continue;
            }
            visited_set.insert(*acyclical_neighbor_id);
            self.visit_node(*acyclical_neighbor_id, sccs, visited_set, cb);
        }

        // now visit the leaf node last
        cb(current_id);
    }

    /// a node is cyclical if the new node to visit is in a SCC of length > 1
    /// OR if it's directly connected to itself
    fn is_cyclical_node(&self, id: Id, sccs: &[Vec<Id>]) -> bool {
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
        let neighbor_ids: Vec<Id> = neighbor_indexes
            .into_iter()
            .map(|neighbor_petgraph_index| {
                *self
                    .petgraph_index_to_id_map
                    .get(&neighbor_petgraph_index)
                    .unwrap()
            })
            .collect();

        neighbor_ids.contains(&id)
    }

    #[allow(dead_code)]
    fn calculate_input_output_len(
        ports_a: Option<&[PortAddress]>,
        ports_b: Option<&[PortAddress]>,
    ) -> usize {
        // make sure outputs is empty and matches length
        const PORT_INDEX_OFFSET: usize = 1;
        let max_a = ports_a.and_then(|output_addresses| {
            output_addresses
                .iter()
                .map(|address| **address.port_id())
                .max()
        });

        let max_b = ports_b.and_then(|output_addresses| {
            output_addresses
                .iter()
                .map(|address| **address.port_id())
                .max()
        });

        match (max_a, max_b) {
            (None, None) => 0,
            (None, Some(max)) => max + PORT_INDEX_OFFSET,
            (Some(max), None) => max + PORT_INDEX_OFFSET,
            (Some(max_a), Some(max_b)) => max_a.max(max_b) + PORT_INDEX_OFFSET,
        }
    }

    /// Returns the number of slots needed to cover all port addresses in `slices`
    /// (i.e. max port_id + 1, or 0 if all slices are empty).
    fn context_slot_count(slices: &[Option<&[PortAddress]>]) -> usize {
        slices
            .iter()
            .filter_map(|opt| *opt)
            .flat_map(|addrs| addrs.iter())
            .map(|addr| **addr.port_id())
            .max()
            .map(|max_id| max_id + 1)
            .unwrap_or(0)
    }

    /// Allocate a zero-filled buffer of `block_size` samples and insert it
    /// into `buffer_pool` under `conn_id`.
    fn allocate_buffer(&mut self, conn_id: ConnectionId) {
        let mut buf: Vec<Sample> = Vec::with_capacity(*self.block_size);
        buf.resize(*self.block_size, Sample::default());
        self.buffer_pool
            .insert(conn_id, core::cell::RefCell::new(buf.into_boxed_slice()));
    }
}

impl GenerateId for Graph {
    fn generate_id(&mut self) -> Id {
        self.id_generator().generate_id()
    }
}

impl crate::traits::Graph for Graph {
    fn add<P: DescribePorts, G: GetPortDescriptors<P>, C: Into<Node> + Deref<Target = G>>(
        &mut self,
        node: C,
    ) -> Result<NodeHandle<P>, GraphAddError> {
        // TODO: check that the adding the node is valid before making it
        // - node id should not already be in the graph
        // - node id should not be weirdly higher than the rest

        let port_descriptors: P = node.get_port_descriptors();
        let node = node.into();
        let node_id = NodeId::from(node.node_id());

        // Build the NodePortMap sized to the node's actual port count BEFORE
        // consuming port_descriptors into NodeHandle.
        // Inputs cover internal input ports, external input ports, and param ports.
        // Outputs cover internal output ports and external output ports.
        let input_slots = Self::context_slot_count(&[
            port_descriptors.input_port_addresses(),
            port_descriptors.external_input_port_addresses(),
            port_descriptors.param_port_addresses(),
        ]);
        let output_slots = Self::context_slot_count(&[
            port_descriptors.output_port_addresses(),
            port_descriptors.external_output_port_addresses(),
        ]);

        let mut inputs: Vec<Option<ConnectionId>> = Vec::with_capacity(input_slots);
        inputs.resize(input_slots, None);
        let mut outputs: Vec<Option<ConnectionId>> = Vec::with_capacity(output_slots);
        outputs.resize(output_slots, None);

        // External output ports are not connected via `connect()`, so we allocate
        // their buffers here and record the connection ID in the port map.
        for ext_addr in port_descriptors
            .external_output_port_addresses()
            .unwrap_or(&[])
        {
            let conn_id = ConnectionId::from(self.id_generator.generate_id());
            self.allocate_buffer(conn_id);
            self.port_address_to_connection_id_map.insert(*ext_addr, conn_id);
            if let Some(slot) = outputs.get_mut(**ext_addr.port_id()) {
                *slot = Some(conn_id);
            }
        }

        // External input ports follow the same pattern (written by the host, read
        // by the node).
        for ext_addr in port_descriptors
            .external_input_port_addresses()
            .unwrap_or(&[])
        {
            let conn_id = ConnectionId::from(self.id_generator.generate_id());
            self.allocate_buffer(conn_id);
            self.port_address_to_connection_id_map.insert(*ext_addr, conn_id);
            if let Some(slot) = inputs.get_mut(**ext_addr.port_id()) {
                *slot = Some(conn_id);
            }
        }

        self.node_port_maps.insert(
            node_id,
            NodePortMap {
                inputs: inputs.into_boxed_slice(),
                outputs: outputs.into_boxed_slice(),
            },
        );

        let node_handle = NodeHandle::new(node_id, port_descriptors);

        // bookkeeping
        let index = self.graph.add_node(node_id);
        self.id_to_pegraph_index_map.insert(*node_id, index);
        self.petgraph_index_to_id_map.insert(index, *node_id);
        // petgraph only keeps ids--we keep the real values for easier bookkeeping
        self.graph_items.insert(*node_id, GraphItem::Node(node));
        // until a node as an outgoing connection, it is a leaf node
        self.leaf_nodes.insert(node_id);

        // must be recomputed on every modification
        // TODO: do incremental updates in the future?
        // May not be necessary, since it can be computed in O(n) time,
        // where n is the number of nodes
        self.visit_order = Some(self.compute_new_visit_order());

        Ok(node_handle)
    }

    fn connect(
        &mut self,
        start_port_address: PortAddress,
        end_port_address: PortAddress,
    ) -> Result<&mut Self, GraphConnectionError> {
        // TODO: check that the connection is valid before making it
        // - connection should not already exist
        // - valid node id, port id, and direction
        // - must be compatible data-types
        // - must be the correct number of connections for both nodes
        // - must be correct node relationship node->node, param->node, etc.

        let connection = Connection::new(self, start_port_address, end_port_address);
        let connection_id = connection.connection_id;

        // Determine the shared data buffer for this connection.
        //
        // For fan-out (one output port connected to multiple inputs), all
        // destinations share the same buffer: only the first `connect` call for
        // a given output port allocates a new buffer; subsequent calls reuse it.
        let is_first_connection_for_output = !self
            .port_address_to_connection_id_map
            .contains_key(&start_port_address);

        // `buffer_owning_conn_id` is the ConnectionId whose entry in `buffer_pool`
        // holds the actual sample data.  For first connections this equals
        // `connection_id`; for fan-out it is the previously registered connection.
        let buffer_owning_conn_id: ConnectionId = if is_first_connection_for_output {
            // First connection from this output port: allocate a fresh buffer.
            self.allocate_buffer(connection_id);
            // Register the output port so later fan-out connections can find it.
            self.port_address_to_connection_id_map
                .insert(start_port_address, connection_id);
            connection_id
        } else {
            // Fan-out: reuse the buffer that was allocated for the first connection.
            *self
                .port_address_to_connection_id_map
                .get(&start_port_address)
                .unwrap()
        };

        // Store the buffer-owning connection_id for the end port.  This is
        // important for fan-out: if the end port is later used as a start port,
        // the pool lookup must find a real buffer.
        self.port_address_to_connection_id_map
            .insert(end_port_address, buffer_owning_conn_id);

        // Update the source node's output port map on the first connection only.
        // For fan-out the connection ID was already recorded.
        if is_first_connection_for_output {
            let start_node_id = start_port_address.node_id();
            if let Some(port_map) = self.node_port_maps.get_mut(&start_node_id) {
                if let Some(slot) = port_map.outputs.get_mut(**start_port_address.port_id()) {
                    *slot = Some(buffer_owning_conn_id);
                }
            }
        }

        // Always update the destination node's input port map.
        let end_node_id = end_port_address.node_id();
        if let Some(port_map) = self.node_port_maps.get_mut(&end_node_id) {
            if let Some(slot) = port_map.inputs.get_mut(**end_port_address.port_id()) {
                *slot = Some(buffer_owning_conn_id);
            }
        }

        let start_node_id = start_port_address.node_id();
        let start_index = *self.id_to_pegraph_index_map.get(&*start_node_id).unwrap();

        let end_node_id = end_port_address.node_id();
        let end_index = *self.id_to_pegraph_index_map.get(&*end_node_id).unwrap();

        self.graph_items
            .insert(*connection_id, GraphItem::Connection(connection));

        self.graph.add_edge(start_index, end_index, connection_id);

        // if it has a connection going out now, it is no longer a leaf node
        self.leaf_nodes.remove(&start_node_id);

        // must be recomputed on every modification
        self.visit_order = Some(self.compute_new_visit_order());

        Ok(self)
    }

    fn run(
        &mut self,
        _inputs: &HashMap<PortAddress, Vec<Sample>>,
        outputs: &mut HashMap<PortAddress, Vec<Sample>>,
    ) -> Result<(), GraphRunError> {
        let Some(ref visit_order) = self.visit_order else {
            return Ok(());
        };

        // Use explicit field borrows so the borrow checker can see that
        // `graph_items` and `node_contexts` are disjoint from each other and
        // from `visit_order` (which borrows `self.visit_order`).
        {
            let graph_items = &mut self.graph_items;
            let node_port_maps = &self.node_port_maps;
            let buffer_pool = &self.buffer_pool;
            let block_size = self.block_size;

            for &id in visit_order.iter() {
                let node = match graph_items.get_mut(&id) {
                    Some(GraphItem::Node(Node::AudioNode(node))) => node,
                    // ParamNodes do not have a process implementation yet; skip them.
                    Some(GraphItem::Node(Node::ParamNode(_))) => continue,
                    _ => return Err(GraphRunError::VisitOrderIncludedNonNodeValue),
                };

                let port_map = node_port_maps
                    .get(&NodeId::from(id))
                    .expect("every node in visit_order must have a NodePortMap");

                let ctx = NodeProcessContext::new(
                    &port_map.inputs,
                    &port_map.outputs,
                    buffer_pool,
                    block_size,
                );

                node.process(ctx)?;
            }
        } // graph_items borrow released here

        // After all nodes have processed, collect the contents of every external
        // output buffer into the caller-provided `outputs` map.  This is the only
        // allocation that happens during `run()`; the DSP loop above is alloc-free.
        for (port_addr, conn_id) in &self.port_address_to_connection_id_map {
            if port_addr.port_address_direction() == PortAddressDirection::ExternalOutput {
                if let Some(cell) = self.buffer_pool.get(conn_id) {
                    let borrowed = cell.borrow();
                    outputs.insert(*port_addr, borrowed.to_vec());
                }
            }
        }

        Ok(())
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

        use crate::{primitives::Id, traits::GetNodeId};

        #[track_caller]
        fn assert_visit_order_matches_handles(
            visit_order: &Option<&[Id]>,
            node_handles: &[Box<dyn GetNodeId>],
        ) {
            let node_handles_as_node_ids: Vec<Id> = node_handles
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
                implementations::{ConstantNode, Graph, MultiplyNode},
                traits::Graph as GraphTrait,
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

                assert_visit_order_matches_handles(
                    &graph.visit_order.as_deref(),
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

            use super::assert_visit_order_matches_handles;
            use crate::{
                implementations::{ConstantNode, Graph, MultiplyNode, OutputNode},
                traits::Graph as GraphTrait,
            };

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

                assert_visit_order_matches_handles(
                    &graph.visit_order.as_deref(),
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

                assert_visit_order_matches_handles(
                    &graph.visit_order.as_deref(),
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

                assert_visit_order_matches_handles(
                    &graph.visit_order.as_deref(),
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
                implementations::{ConstantNode, Graph},
                traits::Graph as GraphTrait,
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

                assert_visit_order_matches_handles(
                    &graph.visit_order.as_deref(),
                    &[Box::new(constant_node_1)],
                );
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

                assert_visit_order_matches_handles(
                    &graph.visit_order.as_deref(),
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

                assert_visit_order_matches_handles(
                    &graph.visit_order.as_deref(),
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

                assert_visit_order_matches_handles(
                    &graph.visit_order.as_deref(),
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

                assert_visit_order_matches_handles(
                    &graph.visit_order.as_deref(),
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

                assert_visit_order_matches_handles(
                    &graph.visit_order.as_deref(),
                    &[Box::new(constant_node_0), Box::new(constant_node_1)],
                );
            }
        }

        mod mix_ayclic_and_cyclic {
            use std::boxed::Box;

            use crate::{
                implementations::{ConstantNode, Graph, MultiplyNode, OutputNode},
                traits::Graph as GraphTrait,
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

                assert_visit_order_matches_handles(
                    &graph.visit_order.as_deref(),
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

            // This graph is totally cracked, and I think the resulting output
            // is jank too, but this test is here mostly just to capture existing
            // behavior and compare to later later implementations if I ever change
            // the implementation.
            //
            // It is somewhat sensible in that it processes all the circular junk before
            // the final 2 nodes, and the inner stuff is correctly linear, but the
            // starting place seems wack, or at least unexpected.
            //
            //       ┌──────────────────────┐        ┌─────┐
            //       │                   ┌──▼────────▼──┐  │
            //       │                   │ Multiply n=0 │  │
            //       │                   └──────┬───────┘  │
            //       │                          └──────────┼─────┐
            // ┌─────┼──────────────────────────┐          │     │
            // │  ┌──┼─────────┐                │          │     │
            // │  │  │  ┌──────▼───────┐ ┌──────▼───────┐  │     │
            // │  │  │  │ Constant n=1 │ │ Constant n=2 │  │     │
            // │  │  │  └──────────┬───┘ └──────────────┘  │     │
            // │  │  │             │          ┌─┴────┐     │     │
            // │  │  │           ┌─▼──────────▼─┐    │     │     │
            // │  │  │           │ Multiply n=3 │    │     │     │
            // │  │  │           └──────┬───────┘    │     │     │
            // │  │  └──────────────────┴────┬───────┼─────┘     │
            // │  │  ┌─────────┐          ┌──▼───────▼───┐       │
            // │  │  │ ┌───────▼──────┐   │ Multiply n=5 │       │
            // │  │  │ │ Constant n=4 │   └──────┬───────┘       │
            // │  │  │ └───────┬──────┘          │               │
            // │  │  └─────────┴─────────────────┼──────┐        │
            // │  └──────────────────────────────┘   ┌──▼────────▼──┐
            // │                                     │ Multiply n=6 │
            // │                                     └───────┬──────┘
            // └─────────────────────────────────────────────┤
            //                                         ┌─────▼──────┐
            //                                         │ Output n=7 │
            //                                         └────────────┘
            #[test]
            fn nuts_recursion() {
                let mut graph = Graph::new();

                let node_0 = MultiplyNode::new(&mut graph);
                let node_1 = ConstantNode::new(&mut graph);
                let node_2 = ConstantNode::new(&mut graph);
                let node_3 = MultiplyNode::new(&mut graph);
                let node_4 = ConstantNode::new(&mut graph);
                let node_5 = MultiplyNode::new(&mut graph);
                let node_6 = MultiplyNode::new(&mut graph);
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
                    .connect(
                        node_0.output_port_address(),
                        node_6.right_operand_input_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        node_1.output_port_address(),
                        node_3.left_operand_input_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        node_2.output_port_address(),
                        node_3.right_operand_input_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        node_2.output_port_address(),
                        node_5.right_operand_input_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        node_3.output_port_address(),
                        node_0.left_operand_input_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        node_3.output_port_address(),
                        node_0.right_operand_input_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        node_3.output_port_address(),
                        node_5.left_operand_input_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        node_4.output_port_address(),
                        node_4.set_constant_value_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(node_4.output_port_address(), node_6.output_port_address())
                    .unwrap();
                graph
                    .connect(
                        node_5.output_port_address(),
                        node_1.set_constant_value_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        node_6.output_port_address(),
                        node_2.set_constant_value_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(node_6.output_port_address(), node_7.input_port_address())
                    .unwrap();

                assert_visit_order_matches_handles(
                    &graph.visit_order.as_deref(),
                    &[
                        Box::new(node_2),
                        Box::new(node_5),
                        Box::new(node_1),
                        Box::new(node_3),
                        Box::new(node_0),
                        Box::new(node_4),
                        Box::new(node_6),
                        Box::new(node_7),
                    ],
                );
            }

            // TODO: add test case for cyclical islands connected by a bridge
            // TODO: add test case for what happens when their priorities are reversed
        }

        mod priority_changes {

            use std::boxed::Box;

            use crate::{
                implementations::{ConstantNode, Graph, MultiplyNode, OutputNode},
                traits::Graph as GraphTrait,
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

                assert_visit_order_matches_handles(
                    &graph.visit_order.as_deref(),
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

    mod audio_processing {
        use hashbrown::HashMap;

        use crate::{
            implementations::{ConstantNode, Graph, OutputNode},
            primitives::Sample,
            traits::Graph as GraphTrait,
        };

        #[test]
        fn only_output_node() {
            let mut graph = Graph::new();
            let output_node = OutputNode::new(&mut graph);
            let output_node = graph.add(output_node).unwrap();

            let inputs = HashMap::new();
            let mut outputs = HashMap::new();
            graph.run(&inputs, &mut outputs).unwrap();

            assert_eq!(
                outputs,
                HashMap::from([(
                    output_node.external_output_port_address(),
                    vec![Sample::default()]
                )])
            )
        }

        #[test]
        fn constant_node_without_value_to_output_node() {
            let mut graph = Graph::new();

            let constant_node = ConstantNode::new(&mut graph);
            let output_node = OutputNode::new(&mut graph);

            let constant_node = graph.add(constant_node).unwrap();
            let output_node = graph.add(output_node).unwrap();

            graph
                .connect(
                    constant_node.output_port_address(),
                    output_node.input_port_address(),
                )
                .unwrap();

            let inputs = HashMap::new();
            let mut outputs = HashMap::new();
            graph.run(&inputs, &mut outputs).unwrap();

            assert_eq!(
                outputs,
                HashMap::from([(
                    output_node.external_output_port_address(),
                    vec![Sample::default()]
                )])
            );
        }

        #[test]
        fn constant_node_with_value_to_output_node() {
            let mut graph = Graph::new();

            let constant_node = ConstantNode::new_with_value(&mut graph, Sample::from(5i32));
            let output_node = OutputNode::new(&mut graph);

            let constant_node = graph.add(constant_node).unwrap();
            let output_node = graph.add(output_node).unwrap();

            graph
                .connect(
                    constant_node.output_port_address(),
                    output_node.input_port_address(),
                )
                .unwrap();

            let inputs = HashMap::new();
            let mut outputs = HashMap::new();
            graph.run(&inputs, &mut outputs).unwrap();

            assert_eq!(
                outputs,
                HashMap::from([(
                    output_node.external_output_port_address(),
                    vec![Sample::from(5i32)]
                )])
            );
        }
    }
}
