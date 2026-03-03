use core::{cell::UnsafeCell, mem::transmute, ops::Deref, ptr::NonNull};

use crate::{
    errors::{BufferAlreadyAllocated, GraphAddError, GraphConnectionError, GraphRunError},
    primitives::{
        BlockSize, BufferPool, Connection, ConnectionId, Id, Node, NodeHandle, NodeId, PortAddress,
        PortId, Sample,
    },
    traits::{AudioNode, DescribePorts, GenerateId, GetNodeId, GetPortDescriptors},
    utils::{IntMap, IntSet, compare_nodes_by_priority},
};

use alloc::{boxed::Box, vec::Vec};
use hashbrown::{HashMap, HashSet};
use petgraph::algo::tarjan_scc;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use wasm_bindgen::prelude::wasm_bindgen;

// TODO: move these implementation-specific structs into a private module
//
/// Per-node storage of which `ConnectionId` backs each port slot.
/// Allocated once at `add()` time; updated during `connect()`.
/// A `None` entry means the port is unconnected.
struct NodeConnectionIdMap {
    // TODO: replace with an IntMap?c
    input_connection_ids: Box<[Option<ConnectionId>]>,
    output_connection_ids: Box<[Option<ConnectionId>]>,
}

/// One entry in the compiled execution plan produced by `Graph::compile`.
///
/// All buffer pointers are extracted once at compile time and reused across `run` calls.
/// External output slots are re-patched from the caller's buffer map each call.
struct CompiledStep {
    /// Raw fat pointer into the `Box<dyn AudioNode>` heap allocation.
    /// Stable because moving a `Box` does not move the heap data it points to.
    node: *mut dyn AudioNode,
    /// Input buffer pointers, one per input port, sized to actual port count.
    /// `None` means the port is unconnected or is a self-loop.
    input_ptrs: Box<[Option<NonNull<[Sample]>>]>,
    /// Output buffer pointers, one per output port, sized to actual port count.
    /// Slots for external ports start as `None` and are patched per `run` call.
    output_ptrs: Box<[Option<NonNull<[Sample]>>]>,
    /// Which output slots are external (caller-supplied) and their `ConnectionId`
    /// so they can be looked up in the caller's output map each `run` call.
    external_output_slots: Box<[(usize, ConnectionId)]>,
    /// Which input slots are external (caller-supplied) and their `ConnectionId`
    /// so they can be looked up in the caller's input map each `run` call.
    external_input_slots: Box<[(usize, ConnectionId)]>,
    block_size: BlockSize,
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
    node_connection_id_map: IntMap<NodeId, NodeConnectionIdMap>,
    id_to_pegraph_index_map: HashMap<Id, petgraph::graph::NodeIndex<petgraph::graph::DefaultIx>>,
    petgraph_index_to_id_map: HashMap<petgraph::graph::NodeIndex<petgraph::graph::DefaultIx>, Id>,
    port_address_to_connection_id_map: HashMap<PortAddress, ConnectionId>,
    graph: petgraph::Graph<NodeId, ConnectionId>,
    leaf_nodes: IntSet<NodeId>,
    block_size: BlockSize,
    buffer_pool: BufferPool,
    /// Lazily compiled flat execution plan. Set to `None` whenever the graph topology changes
    /// (add/connect/disconnect/remove), recompiled on the next `run` call.
    compiled_plan: Option<Vec<CompiledStep>>,
}

impl Graph {
    pub fn with_block_size(block_size: impl Into<BlockSize>) -> Self {
        use crate::utils::IntMap;

        Graph {
            id_generator: GraphIdGenerator::default(),
            graph_items: IntMap::default(),
            node_connection_id_map: IntMap::default(),
            id_to_pegraph_index_map: HashMap::new(),
            graph: petgraph::Graph::<NodeId, ConnectionId>::new(),
            leaf_nodes: IntSet::default(),
            petgraph_index_to_id_map: HashMap::new(),
            port_address_to_connection_id_map: HashMap::new(),
            block_size: block_size.into(),
            buffer_pool: BufferPool::default(),
            compiled_plan: None,
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

    /// Returns the number of port slots needed to hold all ports in the given groups.
    /// With dense port IDs this equals the actual port count; with sparse IDs it overallocates.
    fn count_ports(port_address_groups: &[Option<&[PortAddress]>]) -> usize {
        port_address_groups
            .iter()
            .filter_map(|group| *group)
            .flat_map(|addresses| addresses.iter())
            .map(|address| **address.port_id())
            .max()
            .map(|max_id| max_id + 1)
            .unwrap_or(0)
    }

    /// Checks that all port IDs across the given groups form a dense 0..n sequence.
    /// Called separately for the input group and the output group during `add()`.
    fn validate_dense_port_ids(
        port_address_groups: &[Option<&[PortAddress]>],
    ) -> Result<(), GraphAddError> {
        let mut port_ids: Vec<usize> = port_address_groups
            .iter()
            .filter_map(|group| *group)
            .flat_map(|addresses| addresses.iter())
            .map(|address| **address.port_id())
            .collect();

        port_ids.sort_unstable();

        for (expected_id, &actual_id) in port_ids.iter().enumerate() {
            if actual_id != expected_id {
                return Err(GraphAddError::SparsePortIds {
                    expected: expected_id,
                    actual: actual_id,
                });
            }
        }

        Ok(())
    }

    /// Builds the flat execution plan from the current visit order.
    ///
    /// Extracts raw pointers to each node's heap allocation and to each buffer
    /// in the pool. These pointers are stable until the next topology change
    /// (add/connect/disconnect/remove), which invalidates the plan.
    fn compile(&mut self) -> Vec<CompiledStep> {
        let visit_order = self.compute_new_visit_order();

        let block_size = self.block_size;

        visit_order
            .iter()
            .filter_map(|&id| {
                let node_ptr: *mut dyn AudioNode = {
                    let Some(GraphItem::Node(crate::primitives::Node::AudioNode(node_box))) =
                        self.graph_items.get_mut(&id)
                    else {
                        return None;
                    };
                    // SAFETY: Box heap allocation is stable; moving the Box (e.g. on IntMap
                    // rehash) does not move the heap data the Box points to.
                    &mut **node_box as *mut dyn AudioNode
                };

                let connection_id_map = self.node_connection_id_map.get(&NodeId::from(id))?;

                let mut external_input_slots: Vec<(usize, ConnectionId)> = Vec::new();

                let input_ptrs: Box<[Option<NonNull<[Sample]>>]> = connection_id_map
                    .input_connection_ids
                    .iter()
                    .enumerate()
                    .map(|(slot, connection_id_opt)| {
                        let connection_id = connection_id_opt.as_ref()?;

                        // External input ports have no pool entry; buffers are provided by the
                        // caller at run time.
                        if !self.buffer_pool.contains_key(connection_id) {
                            external_input_slots.push((slot, *connection_id));
                            return None;
                        }

                        // SAFETY: UnsafeCell::get() yields *mut Sample with SRW
                        // (SharedReadWrite) provenance, which lives at the base of the
                        // Stacked Borrows borrow stack and is never invalidated by Unique
                        // retags from mutable accesses in other nodes' process() calls.
                        let raw_ptr: *mut [Sample] = self.buffer_pool.get(connection_id)?.get();
                        Some(unsafe { NonNull::new_unchecked(raw_ptr) })
                    })
                    .collect();

                let mut external_output_slots: Vec<(usize, ConnectionId)> = Vec::new();

                let output_ptrs: Box<[Option<NonNull<[Sample]>>]> = connection_id_map
                    .output_connection_ids
                    .iter()
                    .enumerate()
                    .map(|(slot, connection_id_opt)| {
                        let connection_id = connection_id_opt.as_ref()?;

                        // External output ports are registered in `add()` but have no pool
                        // entry; their buffers are provided by the caller at run time.
                        if !self.buffer_pool.contains_key(connection_id) {
                            external_output_slots.push((slot, *connection_id));
                            return None;
                        }

                        // SAFETY: same SRW provenance argument as input_ptrs above.
                        let raw_ptr: *mut [Sample] = self.buffer_pool.get(connection_id)?.get();
                        Some(unsafe { NonNull::new_unchecked(raw_ptr) })
                    })
                    .collect();

                Some(CompiledStep {
                    node: node_ptr,
                    input_ptrs,
                    output_ptrs,
                    external_output_slots: external_output_slots.into_boxed_slice(),
                    external_input_slots: external_input_slots.into_boxed_slice(),
                    block_size,
                })
            })
            .collect()
    }

    fn ensure_compiled_plan(&mut self) {
        if self.compiled_plan.is_none() {
            self.compiled_plan = Some(self.compile());
        }
    }

    fn allocate_empty_buffer_for_connection(
        &mut self,
        connection_id: ConnectionId,
    ) -> Result<(), BufferAlreadyAllocated> {
        if self.buffer_pool.contains_key(&connection_id) {
            return Err(BufferAlreadyAllocated);
        }

        let mut buf: Vec<Sample> = Vec::with_capacity(*self.block_size);
        buf.resize(*self.block_size, Sample::default());

        // SAFETY: `UnsafeCell<[Sample]>` is `#[repr(transparent)]` over `[Sample]`,
        // so `Box<[Sample]>` and `Box<UnsafeCell<[Sample]>>` have identical layouts.
        let cell_box = unsafe {
            alloc::boxed::Box::from_raw(
                alloc::boxed::Box::into_raw(buf.into_boxed_slice()) as *mut UnsafeCell<[Sample]>
            )
        };

        self.buffer_pool.insert(connection_id, cell_box);

        Ok(())
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

        // Enforce dense per-direction port IDs so that slice lengths equal actual port counts.
        // Regular and external ports share the same slot namespace within each direction.
        Self::validate_dense_port_ids(&[
            port_descriptors.input_port_addresses(),
            port_descriptors.external_input_port_addresses(),
        ])?;
        Self::validate_dense_port_ids(&[
            port_descriptors.output_port_addresses(),
            port_descriptors.external_output_port_addresses(),
        ])?;

        let node = node.into();
        let node_id = NodeId::from(node.node_id());

        // Tracking port information here prevents unnecessary allocation at `run` time.
        let num_input_ports = Self::count_ports(&[
            port_descriptors.input_port_addresses(),
            port_descriptors.external_input_port_addresses(),
        ]);
        let num_output_ports = Self::count_ports(&[
            port_descriptors.output_port_addresses(),
            port_descriptors.external_output_port_addresses(),
        ]);

        let mut input_buffer_connection_ids: Vec<Option<ConnectionId>> =
            Vec::with_capacity(num_input_ports);
        input_buffer_connection_ids.resize(num_input_ports, None);
        let mut output_buffer_connection_ids: Vec<Option<ConnectionId>> =
            Vec::with_capacity(num_output_ports);
        output_buffer_connection_ids.resize(num_output_ports, None);

        let mut external_input_connection_ids: IntMap<PortId, ConnectionId> = IntMap::default();
        let mut external_output_connection_ids: IntMap<PortId, ConnectionId> = IntMap::default();

        // External output ports are not connected via `connect()`, so we only need
        // a ConnectionId for bookkeeping. The actual buffer is provided by the caller at run time.
        for external_output_port_addr in port_descriptors
            .external_output_port_addresses()
            .unwrap_or(&[])
        {
            let port_id = external_output_port_addr.port_id();
            let connection_id = ConnectionId::from(self.id_generator.generate_id());
            external_output_connection_ids.insert(port_id, connection_id);
            self.port_address_to_connection_id_map
                .insert(*external_output_port_addr, connection_id);
            if let Some(connection_id_slot) = output_buffer_connection_ids.get_mut(**port_id) {
                *connection_id_slot = Some(connection_id);
            }
        }

        // External input ports are not connected via `connect()`, so we only need
        // a ConnectionId for bookkeeping. The actual buffer is provided by the caller at run time.
        for external_input_port_addr in port_descriptors
            .external_input_port_addresses()
            .unwrap_or(&[])
        {
            let port_id = external_input_port_addr.port_id();
            let connection_id = ConnectionId::from(self.id_generator.generate_id());
            external_input_connection_ids.insert(port_id, connection_id);
            self.port_address_to_connection_id_map
                .insert(*external_input_port_addr, connection_id);
            if let Some(connection_id_slot) = input_buffer_connection_ids.get_mut(**port_id) {
                *connection_id_slot = Some(connection_id);
            }
        }

        self.node_connection_id_map.insert(
            node_id,
            NodeConnectionIdMap {
                input_connection_ids: input_buffer_connection_ids.into_boxed_slice(),
                output_connection_ids: output_buffer_connection_ids.into_boxed_slice(),
            },
        );

        let node_handle = NodeHandle::new(
            node_id,
            port_descriptors,
            external_input_connection_ids,
            external_output_connection_ids,
        );

        // bookkeeping
        let index = self.graph.add_node(node_id);
        self.id_to_pegraph_index_map.insert(*node_id, index);
        self.petgraph_index_to_id_map.insert(index, *node_id);
        // petgraph only keeps ids--we keep the real values for easier bookkeeping
        self.graph_items.insert(*node_id, GraphItem::Node(node));
        // until a node has an outgoing connection, it is a leaf node
        self.leaf_nodes.insert(node_id);

        // Invalidate cached execution plan; it is recomputed on the next `run()` call.
        self.compiled_plan = None;

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
        // - start port address must be the output of one node and end
        //   address must be the input of another

        let connection = Connection::new(self, start_port_address, end_port_address);
        let connection_id = connection.connection_id;

        self.allocate_empty_buffer_for_connection(connection_id)?;

        self.port_address_to_connection_id_map
            .insert(start_port_address, connection_id);
        self.port_address_to_connection_id_map
            .insert(end_port_address, connection_id);

        let start_node_id = start_port_address.node_id();
        let start_index = *self.id_to_pegraph_index_map.get(&*start_node_id).unwrap();

        // map each start_node's port to its connection_id
        if let Some(port_map) = self.node_connection_id_map.get_mut(&start_node_id)
            && let Some(slot) = port_map
                .output_connection_ids
                .get_mut(**start_port_address.port_id())
        {
            *slot = Some(connection_id);
        }

        let end_node_id = end_port_address.node_id();
        let end_index = *self.id_to_pegraph_index_map.get(&*end_node_id).unwrap();

        // map each end_node's port to its connection_id
        if let Some(port_map) = self.node_connection_id_map.get_mut(&end_node_id)
            && let Some(slot) = port_map
                .input_connection_ids
                .get_mut(**end_port_address.port_id())
        {
            *slot = Some(connection_id);
        }

        self.graph_items
            .insert(*connection_id, GraphItem::Connection(connection));

        self.graph.add_edge(start_index, end_index, connection_id);

        // if it has a connection going out now, it is no longer a leaf node
        self.leaf_nodes.remove(&start_node_id);

        // Invalidate cached execution plan; it is recomputed on the next `run()` call.
        self.compiled_plan = None;

        Ok(self)
    }

    fn run(
        &mut self,
        inputs: &HashMap<ConnectionId, &[Sample]>,
        outputs: &mut HashMap<ConnectionId, &mut [Sample]>,
    ) -> Result<(), GraphRunError> {
        self.ensure_compiled_plan();

        let compiled_plan = self.compiled_plan.as_mut().unwrap();

        for step in compiled_plan.iter_mut() {
            // Patch output slots whose buffers are supplied by the caller for this block.
            for &(slot, connection_id) in step.external_output_slots.iter() {
                step.output_ptrs[slot] = outputs
                    .get_mut(&connection_id)
                    // buffer: &mut &mut [Sample]; &mut **buffer reborrows without moving
                    .map(|buffer| NonNull::from(&mut **buffer));
            }

            // Patch input slots whose buffers are supplied by the caller for this block.
            for &(slot, connection_id) in step.external_input_slots.iter() {
                step.input_ptrs[slot] = inputs.get(&connection_id).map(|&buffer| {
                    // SAFETY: pointer came from a shared reference so it is non-null.
                    unsafe { NonNull::new_unchecked(buffer as *const [Sample] as *mut [Sample]) }
                });
            }

            // SAFETY:
            // 1. Buffer addresses are stable: `allocate_empty_buffer_for_connection` is only
            //    called from `connect()`, never during `run()`, so no heap allocation moves
            //    while `compiled_plan` is in use.  External input/output pointers come from
            //    the caller's slices, which are valid for the duration of `run()`.
            // 2. No mutable aliasing on outputs: each output slot has a unique `ConnectionId`,
            //    so no two `*mut [Sample]` pointers in `output_ptrs` alias the same memory.
            // 3. Visit order enforces exclusive access: by the time a node reads a buffer as
            //    input, the upstream node that writes it has already completed its `process` call.
            // 4. Stacked Borrows / pointer provenance: all pool buffer pointers are derived from
            //    `UnsafeCell::get()` (see `buf_as_raw_slice`), giving them SRW provenance that
            //    is never invalidated by the Unique retags created inside `process()` calls.
            //    see: https://github.com/rust-lang/unsafe-code-guidelines/blob/master/wip/stacked-borrows.md for more info.
            let input_buffers: &[Option<&[Sample]>] =
                unsafe { transmute(step.input_ptrs.as_ref()) };
            let output_buffers: &mut [Option<&mut [Sample]>] =
                unsafe { transmute(step.output_ptrs.as_mut()) };

            // SAFETY: `step.node` points into the heap allocation of a `Box<dyn AudioNode>`
            // stored in `self.graph_items`. Moving the `Box` (e.g. on IntMap rehash) does not
            // move the heap data, so the pointer remains valid. No topology modification
            // (add/connect/disconnect/remove) can occur concurrently with `run()`.
            unsafe { (&mut *step.node).process(input_buffers, output_buffers, step.block_size) }
                .map_err(GraphRunError::AudioNodeRunError)?;
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
            visit_order: &[Id],
            node_handles: &[Box<dyn GetNodeId>],
        ) {
            let node_handles_as_node_ids: Vec<Id> = node_handles
                .iter()
                .map(|node_handle| node_handle.node_id())
                .collect();
            assert_eq!(
                visit_order,
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

                let visit_order = graph.compute_new_visit_order();
                assert_visit_order_matches_handles(
                    visit_order.as_slice(),
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

                let constant_node_handle = graph.add(constant_node).unwrap();
                let multiply_node_handle = graph.add(multiply_node).unwrap();

                graph
                    .connect(
                        constant_node_handle.output_port_address(),
                        multiply_node_handle.left_operand_input_address(),
                    )
                    .unwrap();

                let visit_order = graph.compute_new_visit_order();
                assert_visit_order_matches_handles(
                    visit_order.as_slice(),
                    &[
                        Box::new(constant_node_handle),
                        Box::new(multiply_node_handle),
                    ],
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

                let node_0_handle = graph.add(node_0).unwrap();
                let node_1_handle = graph.add(node_1).unwrap();
                let node_2_handle = graph.add(node_2).unwrap();
                let node_3_handle = graph.add(node_3).unwrap();
                let node_4_handle = graph.add(node_4).unwrap();

                graph
                    .connect(
                        node_0_handle.output_port_address(),
                        node_2_handle.left_operand_input_address(),
                    )
                    .unwrap()
                    .connect(
                        node_1_handle.output_port_address(),
                        node_2_handle.right_operand_input_address(),
                    )
                    .unwrap()
                    .connect(
                        node_2_handle.output_port_address(),
                        node_4_handle.left_operand_input_address(),
                    )
                    .unwrap()
                    .connect(
                        node_3_handle.output_port_address(),
                        node_4_handle.right_operand_input_address(),
                    )
                    .unwrap();

                let visit_order = graph.compute_new_visit_order();
                assert_visit_order_matches_handles(
                    visit_order.as_slice(),
                    &[
                        Box::new(node_0_handle),
                        Box::new(node_1_handle),
                        Box::new(node_2_handle),
                        Box::new(node_3_handle),
                        Box::new(node_4_handle),
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

                let node_0_handle = graph.add(node_0).unwrap();
                let node_1_handle = graph.add(node_1).unwrap();
                let node_2_handle = graph.add(node_2).unwrap();
                let node_3_handle = graph.add(node_3).unwrap();
                let node_4_handle = graph.add(node_4).unwrap();

                graph
                    .connect(
                        node_0_handle.output_port_address(),
                        node_1_handle.input_port_address(),
                    )
                    .unwrap()
                    .connect(
                        node_0_handle.output_port_address(),
                        node_2_handle.input_port_address(),
                    )
                    .unwrap()
                    .connect(
                        node_0_handle.output_port_address(),
                        node_3_handle.input_port_address(),
                    )
                    .unwrap()
                    .connect(
                        node_0_handle.output_port_address(),
                        node_4_handle.input_port_address(),
                    )
                    .unwrap();

                let visit_order = graph.compute_new_visit_order();
                assert_visit_order_matches_handles(
                    visit_order.as_slice(),
                    &[
                        Box::new(node_0_handle),
                        Box::new(node_1_handle),
                        Box::new(node_2_handle),
                        Box::new(node_3_handle),
                        Box::new(node_4_handle),
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
                let constant_node_1_handle = graph.add(constant_node_1).unwrap();

                graph
                    .connect(
                        constant_node_1_handle.output_port_address(),
                        constant_node_1_handle.set_constant_value_port_address(),
                    )
                    .unwrap();

                let visit_order = graph.compute_new_visit_order();
                assert_visit_order_matches_handles(
                    visit_order.as_slice(),
                    &[Box::new(constant_node_1_handle)],
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

                let constant_node_1_handle = graph.add(constant_node_1).unwrap();
                let constant_node_2_handle = graph.add(constant_node_2).unwrap();

                graph
                    .connect(
                        constant_node_1_handle.output_port_address(),
                        constant_node_2_handle.set_constant_value_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        constant_node_2_handle.output_port_address(),
                        constant_node_1_handle.set_constant_value_port_address(),
                    )
                    .unwrap();

                let visit_order = graph.compute_new_visit_order();
                assert_visit_order_matches_handles(
                    visit_order.as_slice(),
                    &[
                        Box::new(constant_node_1_handle),
                        Box::new(constant_node_2_handle),
                    ],
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

                let constant_node_0_handle = graph.add(constant_node_0).unwrap();
                let constant_node_1_handle = graph.add(constant_node_1).unwrap();
                let constant_node_2_handle = graph.add(constant_node_2).unwrap();

                graph
                    .connect(
                        constant_node_0_handle.output_port_address(),
                        constant_node_1_handle.set_constant_value_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        constant_node_1_handle.output_port_address(),
                        constant_node_2_handle.set_constant_value_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        constant_node_2_handle.output_port_address(),
                        constant_node_0_handle.set_constant_value_port_address(),
                    )
                    .unwrap();

                let visit_order = graph.compute_new_visit_order();
                assert_visit_order_matches_handles(
                    visit_order.as_slice(),
                    &[
                        Box::new(constant_node_0_handle),
                        Box::new(constant_node_1_handle),
                        Box::new(constant_node_2_handle),
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

                let constant_node_0_handle = graph.add(constant_node_0).unwrap();
                let constant_node_1_handle = graph.add(constant_node_1).unwrap();

                graph
                    .connect(
                        constant_node_0_handle.output_port_address(),
                        constant_node_1_handle.set_constant_value_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        constant_node_1_handle.output_port_address(),
                        constant_node_1_handle.set_constant_value_port_address(),
                    )
                    .unwrap();

                let visit_order = graph.compute_new_visit_order();
                assert_visit_order_matches_handles(
                    visit_order.as_slice(),
                    &[
                        Box::new(constant_node_0_handle),
                        Box::new(constant_node_1_handle),
                    ],
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

                let constant_node_0_handle = graph.add(constant_node_0).unwrap();
                let constant_node_1_handle = graph.add(constant_node_1).unwrap();

                graph
                    .connect(
                        constant_node_0_handle.output_port_address(),
                        constant_node_0_handle.set_constant_value_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        constant_node_0_handle.output_port_address(),
                        constant_node_1_handle.set_constant_value_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        constant_node_1_handle.output_port_address(),
                        constant_node_1_handle.set_constant_value_port_address(),
                    )
                    .unwrap();

                let visit_order = graph.compute_new_visit_order();
                assert_visit_order_matches_handles(
                    visit_order.as_slice(),
                    &[
                        Box::new(constant_node_0_handle),
                        Box::new(constant_node_1_handle),
                    ],
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

                let constant_node_0_handle = graph.add(constant_node_0).unwrap();
                let constant_node_1_handle = graph.add(constant_node_1).unwrap();

                graph
                    .connect(
                        constant_node_0_handle.output_port_address(),
                        constant_node_0_handle.set_constant_value_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        constant_node_0_handle.output_port_address(),
                        constant_node_1_handle.set_constant_value_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        constant_node_1_handle.output_port_address(),
                        constant_node_1_handle.set_constant_value_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        constant_node_1_handle.output_port_address(),
                        constant_node_0_handle.set_constant_value_port_address(),
                    )
                    .unwrap();

                let visit_order = graph.compute_new_visit_order();
                assert_visit_order_matches_handles(
                    visit_order.as_slice(),
                    &[
                        Box::new(constant_node_0_handle),
                        Box::new(constant_node_1_handle),
                    ],
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

                let node_0_handle = graph.add(node_0).unwrap();
                let node_1_handle = graph.add(node_1).unwrap();
                let node_2_handle = graph.add(node_2).unwrap();
                let node_3_handle = graph.add(node_3).unwrap();
                let node_4_handle = graph.add(node_4).unwrap();
                let node_5_handle = graph.add(node_5).unwrap();
                let node_6_handle = graph.add(node_6).unwrap();
                let node_7_handle = graph.add(node_7).unwrap();

                graph
                    .connect(
                        node_1_handle.output_port_address(),
                        node_2_handle.input_port_address(),
                    )
                    .unwrap()
                    .connect(
                        node_3_handle.output_port_address(),
                        node_5_handle.left_operand_input_address(),
                    )
                    .unwrap()
                    .connect(
                        node_4_handle.output_port_address(),
                        node_5_handle.right_operand_input_address(),
                    )
                    .unwrap()
                    .connect(
                        node_5_handle.output_port_address(),
                        node_3_handle.set_constant_value_port_address(),
                    )
                    .unwrap()
                    .connect(
                        node_4_handle.output_port_address(),
                        node_4_handle.set_constant_value_port_address(),
                    )
                    .unwrap()
                    .connect(
                        node_5_handle.output_port_address(),
                        node_6_handle.input_port_address(),
                    )
                    .unwrap();

                let visit_order = graph.compute_new_visit_order();
                assert_visit_order_matches_handles(
                    visit_order.as_slice(),
                    &[
                        Box::new(node_0_handle),
                        Box::new(node_1_handle),
                        Box::new(node_2_handle),
                        Box::new(node_3_handle),
                        Box::new(node_4_handle),
                        Box::new(node_5_handle),
                        Box::new(node_6_handle),
                        Box::new(node_7_handle),
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

                let node_0_handle = graph.add(node_0).unwrap();
                let node_1_handle = graph.add(node_1).unwrap();
                let node_2_handle = graph.add(node_2).unwrap();
                let node_3_handle = graph.add(node_3).unwrap();
                let node_4_handle = graph.add(node_4).unwrap();
                let node_5_handle = graph.add(node_5).unwrap();
                let node_6_handle = graph.add(node_6).unwrap();
                let node_7_handle = graph.add(node_7).unwrap();

                graph
                    .connect(
                        node_0_handle.output_port_address(),
                        node_6_handle.right_operand_input_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        node_1_handle.output_port_address(),
                        node_3_handle.left_operand_input_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        node_2_handle.output_port_address(),
                        node_3_handle.right_operand_input_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        node_2_handle.output_port_address(),
                        node_5_handle.right_operand_input_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        node_3_handle.output_port_address(),
                        node_0_handle.left_operand_input_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        node_3_handle.output_port_address(),
                        node_0_handle.right_operand_input_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        node_3_handle.output_port_address(),
                        node_5_handle.left_operand_input_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        node_4_handle.output_port_address(),
                        node_4_handle.set_constant_value_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        node_4_handle.output_port_address(),
                        node_6_handle.output_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        node_5_handle.output_port_address(),
                        node_1_handle.set_constant_value_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        node_6_handle.output_port_address(),
                        node_2_handle.set_constant_value_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        node_6_handle.output_port_address(),
                        node_7_handle.input_port_address(),
                    )
                    .unwrap();

                let visit_order = graph.compute_new_visit_order();
                assert_visit_order_matches_handles(
                    visit_order.as_slice(),
                    &[
                        Box::new(node_2_handle),
                        Box::new(node_5_handle),
                        Box::new(node_1_handle),
                        Box::new(node_3_handle),
                        Box::new(node_0_handle),
                        Box::new(node_4_handle),
                        Box::new(node_6_handle),
                        Box::new(node_7_handle),
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
                let node_2_handle = graph.add(node_2).unwrap();
                let node_6_handle = graph.add(node_6).unwrap();
                let node_4_handle = graph.add(node_4).unwrap();
                let node_5_handle = graph.add(node_5).unwrap();
                let node_3_handle = graph.add(node_3).unwrap();
                let node_0_handle = graph.add(node_0).unwrap();
                let node_7_handle = graph.add(node_7).unwrap();
                let node_1_handle = graph.add(node_1).unwrap();

                // connect them in weird order
                graph
                    .connect(
                        node_5_handle.output_port_address(),
                        node_4_handle.input_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        node_2_handle.output_port_address(),
                        node_6_handle.left_operand_input_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        node_6_handle.output_port_address(),
                        node_0_handle.input_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        node_7_handle.output_port_address(),
                        node_6_handle.right_operand_input_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        node_7_handle.output_port_address(),
                        node_7_handle.set_constant_value_port_address(),
                    )
                    .unwrap();
                graph
                    .connect(
                        node_6_handle.output_port_address(),
                        node_2_handle.set_constant_value_port_address(),
                    )
                    .unwrap();

                let visit_order = graph.compute_new_visit_order();
                assert_visit_order_matches_handles(
                    visit_order.as_slice(),
                    &[
                        Box::new(node_2_handle),
                        Box::new(node_7_handle),
                        Box::new(node_6_handle),
                        Box::new(node_0_handle),
                        Box::new(node_1_handle),
                        Box::new(node_3_handle),
                        Box::new(node_5_handle),
                        Box::new(node_4_handle),
                    ],
                );
            }
        }
    }

    mod audio_processing {
        use alloc::vec::Vec;
        use hashbrown::HashMap;

        use crate::{
            implementations::{
                ConstantNode, Graph, MultiplyNode, OutputNode, OutputNodePortDescriptors,
            },
            primitives::Sample,
            traits::Graph as GraphTrait,
        };

        /// Converts a slice of `f32` literals into `Vec<Sample>` for concise assertions.
        fn samples(values: &[f32]) -> Vec<Sample> {
            values.iter().map(|&v| Sample::from(v)).collect()
        }

        /// Returns the single external output `ConnectionId` from an `OutputNode` handle.
        macro_rules! ext_output_id {
            ($handle:expr) => {
                *$handle
                    .external_output_connection_ids()
                    .get(&OutputNodePortDescriptors::EXTERNAL_OUTPUT_PORT_ID)
                    .unwrap()
            };
        }

        #[test]
        fn only_output_node() {
            let mut graph = Graph::new();
            let output_node = OutputNode::new(&mut graph);
            let output_node_handle = graph.add(output_node).unwrap();
            let &external_output_connection_id = output_node_handle
                .external_output_connection_ids()
                .get(&OutputNodePortDescriptors::EXTERNAL_OUTPUT_PORT_ID)
                .unwrap();

            let inputs = HashMap::new();
            let mut output_buffer = vec![Sample::default()];
            let mut outputs =
                HashMap::from([(external_output_connection_id, output_buffer.as_mut_slice())]);
            graph.run(&inputs, &mut outputs).unwrap();

            assert_eq!(
                outputs,
                HashMap::from([(
                    external_output_connection_id,
                    vec![Sample::default()].as_mut_slice(),
                )])
            )
        }

        #[test]
        fn no_external_output_supplied_none_written() {
            let mut graph = Graph::new();

            let constant_node = ConstantNode::new(&mut graph);
            let output_node = OutputNode::new(&mut graph);

            let constant_node_handle = graph.add(constant_node).unwrap();
            let output_node_handle = graph.add(output_node).unwrap();

            graph
                .connect(
                    constant_node_handle.output_port_address(),
                    output_node_handle.input_port_address(),
                )
                .unwrap();

            let inputs = HashMap::new();
            let mut outputs = HashMap::from([]);
            graph.run(&inputs, &mut outputs).unwrap();

            assert_eq!(outputs, HashMap::from([]));
        }

        #[test]
        fn constant_node_with_value_to_output_node() {
            let mut graph = Graph::new();

            let expected_sample_value = 5i32;
            let constant_node =
                ConstantNode::new_with_value(&mut graph, Sample::from(expected_sample_value));
            let output_node = OutputNode::new(&mut graph);

            let constant_node_handle = graph.add(constant_node).unwrap();
            let output_node_handle = graph.add(output_node).unwrap();
            let &external_output_connection_id = output_node_handle
                .external_output_connection_ids()
                .get(&OutputNodePortDescriptors::EXTERNAL_OUTPUT_PORT_ID)
                .unwrap();

            graph
                .connect(
                    constant_node_handle.output_port_address(),
                    output_node_handle.input_port_address(),
                )
                .unwrap();

            let inputs = HashMap::new();
            let mut output_buffer = vec![Sample::default()];
            let mut outputs =
                HashMap::from([(external_output_connection_id, output_buffer.as_mut_slice())]);
            graph.run(&inputs, &mut outputs).unwrap();

            assert_eq!(
                outputs,
                HashMap::from([(
                    external_output_connection_id,
                    vec![Sample::from(expected_sample_value)].as_mut_slice(),
                )])
            );
        }

        #[test]
        fn empty_graph_run_succeeds() {
            let mut graph = Graph::new();
            let inputs = HashMap::new();
            let mut outputs = HashMap::new();
            assert!(graph.run(&inputs, &mut outputs).is_ok());
        }

        #[test]
        fn multiply_node_produces_product_of_two_constants() {
            // const(2.0) ─left─┐
            //                  ├─ multiply ─── output
            // const(3.0) ─right┘
            let mut graph = Graph::new();
            let const_2 = ConstantNode::new_with_value(&mut graph, 2.0f32);
            let const_3 = ConstantNode::new_with_value(&mut graph, 3.0f32);
            let multiply = MultiplyNode::new(&mut graph);
            let output = OutputNode::new(&mut graph);

            let const_2_handle = graph.add(const_2).unwrap();
            let const_3_handle = graph.add(const_3).unwrap();
            let multiply_handle = graph.add(multiply).unwrap();
            let output_handle = graph.add(output).unwrap();
            let ext_id = ext_output_id!(output_handle);

            graph
                .connect(
                    const_2_handle.output_port_address(),
                    multiply_handle.left_operand_input_address(),
                )
                .unwrap()
                .connect(
                    const_3_handle.output_port_address(),
                    multiply_handle.right_operand_input_address(),
                )
                .unwrap()
                .connect(
                    multiply_handle.output_port_address(),
                    output_handle.input_port_address(),
                )
                .unwrap();

            let inputs = HashMap::new();
            let mut out_buf = vec![Sample::default()];
            let mut outputs = HashMap::from([(ext_id, out_buf.as_mut_slice())]);
            graph.run(&inputs, &mut outputs).unwrap();

            assert_eq!(outputs[&ext_id], samples(&[6.0]).as_slice());
        }

        // TODO: fan-out is not yet implemented.
        //
        // Each `connect()` call overwrites the source node's `output_connection_ids` slot
        // with the newest `ConnectionId`, so only the most-recently-connected consumer's buffer
        // is ever written.  The fix is to detect fan-out in `connect()` and reuse the first
        // buffer's `ConnectionId` for all subsequent consumers (sharing the pool buffer).
        #[test]
        #[ignore = "fan-out not yet implemented: only the last-connected consumer receives audio"]
        fn fan_out_one_constant_to_multiple_outputs() {
            // const(5.0) ─── output_1
            //            └── output_2
            let mut graph = Graph::new();
            let constant = ConstantNode::new_with_value(&mut graph, 5.0f32);
            let output_1 = OutputNode::new(&mut graph);
            let output_2 = OutputNode::new(&mut graph);

            let constant_handle = graph.add(constant).unwrap();
            let output_1_handle = graph.add(output_1).unwrap();
            let output_2_handle = graph.add(output_2).unwrap();
            let ext_id_1 = ext_output_id!(output_1_handle);
            let ext_id_2 = ext_output_id!(output_2_handle);

            graph
                .connect(
                    constant_handle.output_port_address(),
                    output_1_handle.input_port_address(),
                )
                .unwrap()
                .connect(
                    constant_handle.output_port_address(),
                    output_2_handle.input_port_address(),
                )
                .unwrap();

            let inputs = HashMap::new();
            let mut out_buf_1 = vec![Sample::default()];
            let mut out_buf_2 = vec![Sample::default()];
            let mut outputs = HashMap::from([
                (ext_id_1, out_buf_1.as_mut_slice()),
                (ext_id_2, out_buf_2.as_mut_slice()),
            ]);
            graph.run(&inputs, &mut outputs).unwrap();

            assert_eq!(outputs[&ext_id_1], samples(&[5.0]).as_slice());
            assert_eq!(outputs[&ext_id_2], samples(&[5.0]).as_slice());
        }

        #[test]
        fn larger_block_size_fills_all_samples() {
            let block_size = 4usize;
            let mut graph = Graph::with_block_size(block_size);
            let constant = ConstantNode::new_with_value(&mut graph, 9.0f32);
            let output = OutputNode::new(&mut graph);

            let constant_handle = graph.add(constant).unwrap();
            let output_handle = graph.add(output).unwrap();
            let ext_id = ext_output_id!(output_handle);

            graph
                .connect(
                    constant_handle.output_port_address(),
                    output_handle.input_port_address(),
                )
                .unwrap();

            let inputs = HashMap::new();
            let mut out_buf = vec![Sample::default(); block_size];
            let mut outputs = HashMap::from([(ext_id, out_buf.as_mut_slice())]);
            graph.run(&inputs, &mut outputs).unwrap();

            assert_eq!(outputs[&ext_id], samples(&[9.0, 9.0, 9.0, 9.0]).as_slice());
        }

        #[test]
        fn multiple_run_calls_produce_consistent_results() {
            let mut graph = Graph::new();
            let constant = ConstantNode::new_with_value(&mut graph, 3.0f32);
            let output = OutputNode::new(&mut graph);

            let constant_handle = graph.add(constant).unwrap();
            let output_handle = graph.add(output).unwrap();
            let ext_id = ext_output_id!(output_handle);

            graph
                .connect(
                    constant_handle.output_port_address(),
                    output_handle.input_port_address(),
                )
                .unwrap();

            for _ in 0..3 {
                let inputs = HashMap::new();
                let mut out_buf = vec![Sample::default()];
                let mut outputs = HashMap::from([(ext_id, out_buf.as_mut_slice())]);
                graph.run(&inputs, &mut outputs).unwrap();
                assert_eq!(outputs[&ext_id], samples(&[3.0]).as_slice());
            }
        }

        #[test]
        fn chained_multiply_nodes() {
            // const(2) ─left─┐
            //                ├─ multiply_1 ─left─┐
            // const(3) ─right┘                   ├─ multiply_2 ─── output
            //                         const(4) ─right┘
            // Expected: (2 * 3) * 4 = 24
            let mut graph = Graph::new();
            let const_2 = ConstantNode::new_with_value(&mut graph, 2.0f32);
            let const_3 = ConstantNode::new_with_value(&mut graph, 3.0f32);
            let const_4 = ConstantNode::new_with_value(&mut graph, 4.0f32);
            let multiply_1 = MultiplyNode::new(&mut graph);
            let multiply_2 = MultiplyNode::new(&mut graph);
            let output = OutputNode::new(&mut graph);

            let const_2_handle = graph.add(const_2).unwrap();
            let const_3_handle = graph.add(const_3).unwrap();
            let const_4_handle = graph.add(const_4).unwrap();
            let multiply_1_handle = graph.add(multiply_1).unwrap();
            let multiply_2_handle = graph.add(multiply_2).unwrap();
            let output_handle = graph.add(output).unwrap();
            let ext_id = ext_output_id!(output_handle);

            graph
                .connect(
                    const_2_handle.output_port_address(),
                    multiply_1_handle.left_operand_input_address(),
                )
                .unwrap()
                .connect(
                    const_3_handle.output_port_address(),
                    multiply_1_handle.right_operand_input_address(),
                )
                .unwrap()
                .connect(
                    multiply_1_handle.output_port_address(),
                    multiply_2_handle.left_operand_input_address(),
                )
                .unwrap()
                .connect(
                    const_4_handle.output_port_address(),
                    multiply_2_handle.right_operand_input_address(),
                )
                .unwrap()
                .connect(
                    multiply_2_handle.output_port_address(),
                    output_handle.input_port_address(),
                )
                .unwrap();

            let inputs = HashMap::new();
            let mut out_buf = vec![Sample::default()];
            let mut outputs = HashMap::from([(ext_id, out_buf.as_mut_slice())]);
            graph.run(&inputs, &mut outputs).unwrap();

            assert_eq!(outputs[&ext_id], samples(&[24.0]).as_slice());
        }

        #[test]
        fn multiply_node_with_unconnected_inputs_outputs_zero() {
            // A multiply node with no connections defaults to 0.0 * 0.0 = 0.0
            let mut graph = Graph::new();
            let multiply = MultiplyNode::new(&mut graph);
            let output = OutputNode::new(&mut graph);

            let multiply_handle = graph.add(multiply).unwrap();
            let output_handle = graph.add(output).unwrap();
            let ext_id = ext_output_id!(output_handle);

            graph
                .connect(
                    multiply_handle.output_port_address(),
                    output_handle.input_port_address(),
                )
                .unwrap();

            let inputs = HashMap::new();
            let mut out_buf = vec![Sample::default()];
            let mut outputs = HashMap::from([(ext_id, out_buf.as_mut_slice())]);
            graph.run(&inputs, &mut outputs).unwrap();

            assert_eq!(outputs[&ext_id], samples(&[0.0]).as_slice());
        }

        mod external_inputs {
            use core::ops::Deref;

            use hashbrown::HashMap;

            use crate::traits::Graph as GraphTrait;
            use crate::{
                errors::AudioNodeRunError,
                implementations::Graph,
                primitives::{
                    BlockSize, Id, NodeId, PortAddress, PortAddressDirection, PortId, Priority,
                    Sample,
                },
                traits::{
                    Audio, AudioNode, DescribePorts, GenerateId, GetNodeId, GetPortDescriptors,
                    GetPriority,
                },
            };

            /// A node with one external input and one external output that copies
            /// the caller-supplied input buffer to the caller-supplied output buffer.
            struct PassthroughNode {
                node_id: NodeId,
                port_descriptors: PassthroughPortDescriptors,
            }

            impl PassthroughNode {
                fn new<G: GenerateId>(id_gen: &mut G) -> Audio<Self> {
                    let node_id = NodeId::from(id_gen.generate_id());
                    Audio(Self {
                        node_id,
                        port_descriptors: PassthroughPortDescriptors::new(node_id),
                    })
                }
            }

            impl GetNodeId for PassthroughNode {
                fn node_id(&self) -> Id {
                    *self.node_id
                }
            }

            impl GetPriority for PassthroughNode {
                fn get_priority(&self) -> Priority {
                    (**self.node_id).into()
                }
            }

            impl Deref for PassthroughNode {
                type Target = PassthroughPortDescriptors;
                fn deref(&self) -> &Self::Target {
                    &self.port_descriptors
                }
            }

            impl AudioNode for PassthroughNode {
                fn process(
                    &mut self,
                    inputs: &[Option<&[Sample]>],
                    outputs: &mut [Option<&mut [Sample]>],
                    _block_size: BlockSize,
                ) -> Result<(), AudioNodeRunError> {
                    let Some(out) = outputs[0].as_deref_mut() else {
                        return Ok(());
                    };
                    let input = inputs[0].unwrap_or(&[]);
                    for (o, &i) in out.iter_mut().zip(input.iter()) {
                        *o = i;
                    }
                    Ok(())
                }
            }

            #[derive(Copy, Clone)]
            struct PassthroughPortDescriptors {
                external_input: [PortAddress; 1],
                external_output: [PortAddress; 1],
            }

            impl PassthroughPortDescriptors {
                const EXTERNAL_INPUT_PORT_ID: PortId = PortId::new(0);
                const EXTERNAL_OUTPUT_PORT_ID: PortId = PortId::new(0);

                fn new(node_id: NodeId) -> Self {
                    Self {
                        external_input: [PortAddress::new(
                            node_id,
                            Self::EXTERNAL_INPUT_PORT_ID,
                            PortAddressDirection::ExternalInput,
                        )],
                        external_output: [PortAddress::new(
                            node_id,
                            Self::EXTERNAL_OUTPUT_PORT_ID,
                            PortAddressDirection::ExternalOutput,
                        )],
                    }
                }
            }

            impl DescribePorts for PassthroughPortDescriptors {
                fn external_input_port_addresses(&self) -> Option<&[PortAddress]> {
                    Some(&self.external_input)
                }

                fn external_output_port_addresses(&self) -> Option<&[PortAddress]> {
                    Some(&self.external_output)
                }
            }

            impl GetPortDescriptors<PassthroughPortDescriptors> for PassthroughNode {
                fn get_port_descriptors(&self) -> PassthroughPortDescriptors {
                    self.port_descriptors
                }
            }

            // ┌─────────────┐
            // │ Passthrough │  (external in → external out)
            // └─────────────┘
            #[test]
            fn caller_input_is_passed_through_to_output() {
                let block_size = 4usize;
                let mut graph = Graph::with_block_size(block_size);
                let node = PassthroughNode::new(&mut graph);
                let handle = graph.add(node).unwrap();

                let &ext_input_conn_id = handle
                    .external_input_connection_ids()
                    .get(&PassthroughPortDescriptors::EXTERNAL_INPUT_PORT_ID)
                    .unwrap();
                let &ext_output_conn_id = handle
                    .external_output_connection_ids()
                    .get(&PassthroughPortDescriptors::EXTERNAL_OUTPUT_PORT_ID)
                    .unwrap();

                let input_data = [
                    Sample::from(1.0f32),
                    Sample::from(2.0f32),
                    Sample::from(3.0f32),
                    Sample::from(4.0f32),
                ];
                let inputs = HashMap::from([(ext_input_conn_id, input_data.as_slice())]);
                let mut output_buffer = vec![Sample::default(); block_size];
                let mut outputs =
                    HashMap::from([(ext_output_conn_id, output_buffer.as_mut_slice())]);

                graph.run(&inputs, &mut outputs).unwrap();

                assert_eq!(outputs[&ext_output_conn_id], input_data.as_slice());
            }
        }
    }
}
