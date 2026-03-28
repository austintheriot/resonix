use core::{cell::UnsafeCell, mem::transmute, ops::Deref, ptr::NonNull};

use crate::implementations::{AudioBuffer, AudioBufferMut, RawAudioBuffer};
use crate::primitives::CurrentTime;
use crate::traits::AudioNode;
use crate::{
    errors::{BufferAlreadyAllocated, GraphAddError, GraphConnectionError, GraphRunError},
    primitives::{
        BlockSize, ChannelledBuffer, Connection, ConnectionId, ExternalConnectionId, Id,
        NodeHandle, NodeId, PortAddress, PortAddressDirection, PortDescriptor, Sample,
    },
    traits::{DescribePorts, GenerateId, GetPortDescriptors},
    utils::{IntMap, IntSet, compare_nodes_by_priority},
};

mod compiled_step;
mod erased_audio_node;
mod graph_id_generator;
mod graph_item;
mod node;
mod node_connection_id_map;

use compiled_step::*;
use erased_audio_node::*;
use graph_id_generator::*;
use graph_item::*;
use node::*;
use node_connection_id_map::*;

use alloc::{boxed::Box, vec, vec::Vec};
use hashbrown::{HashMap, HashSet};
use petgraph::algo::tarjan_scc;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Graph {
    id_generator: GraphIdGenerator,
    graph_items: IntMap<Id, GraphItem>,
    node_connection_id_map: IntMap<NodeId, NodeConnectionIdMap>,
    node_id_to_petgraph_index: HashMap<Id, petgraph::graph::NodeIndex<petgraph::graph::DefaultIx>>,
    petgraph_index_to_node_id: HashMap<petgraph::graph::NodeIndex<petgraph::graph::DefaultIx>, Id>,
    port_address_to_connection_id_map: HashMap<PortAddress, ConnectionId>,
    /// Channel counts for all registered port addresses (both internal and external).
    port_address_to_channel_count: HashMap<PortAddress, usize>,
    graph: petgraph::Graph<NodeId, ConnectionId>,
    leaf_nodes: IntSet<NodeId>,
    block_size: BlockSize,
    buffer_pool: crate::primitives::BufferPool,
    /// Lazily compiled flat execution plan. Set to `None` whenever the graph topology changes
    /// (add/connect/disconnect/remove), recompiled on the next `run` call.
    compiled_plan: Option<Vec<CompiledStep>>,
}

// TODO: delete. Just using to test wasm imports for now on web
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Graph {
    #[wasm_bindgen]
    pub fn test_with_block_size(block_size: usize) -> Graph {
        Self::with_block_size(BlockSize::from(block_size))
    }
}

impl Graph {
    pub fn with_block_size(block_size: impl Into<BlockSize>) -> Self {
        use crate::utils::IntMap;

        Graph {
            id_generator: GraphIdGenerator::default(),
            graph_items: IntMap::default(),
            node_connection_id_map: IntMap::default(),
            node_id_to_petgraph_index: HashMap::new(),
            graph: petgraph::Graph::<NodeId, ConnectionId>::new(),
            leaf_nodes: IntSet::default(),
            petgraph_index_to_node_id: HashMap::new(),
            port_address_to_connection_id_map: HashMap::new(),
            port_address_to_channel_count: HashMap::new(),
            block_size: block_size.into(),
            buffer_pool: crate::primitives::BufferPool::default(),
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

    fn invalidate_compiled_plan(&mut self) {
        self.compiled_plan = None;
    }

    /// Adds a node to the internal petgraph and keeps both index lookup maps in sync.
    fn register_node_in_petgraph(&mut self, node_id: NodeId) {
        let petgraph_index = self.graph.add_node(node_id);
        self.node_id_to_petgraph_index
            .insert(*node_id, petgraph_index);
        self.petgraph_index_to_node_id
            .insert(petgraph_index, *node_id);
    }

    /// Assigns `ExternalConnectionId`s to external ports within a combined port direction slice.
    ///
    /// Iterates all `port_descriptors` and fills `external_slots` (indexed by combined PortId,
    /// same length as the combined port slot array) with `Some(external_connection_id)` at each position whose
    /// address direction is `ExternalInput` or `ExternalOutput`. Internal ports are skipped.
    fn register_external_port_addresses(
        &mut self,
        port_descriptors: &[PortDescriptor],
        external_slots: &mut [Option<ExternalConnectionId>],
    ) {
        for descriptor in port_descriptors {
            if !matches!(
                descriptor.address.port_address_direction(),
                PortAddressDirection::ExternalInput | PortAddressDirection::ExternalOutput,
            ) {
                continue;
            }
            let slot_index = **descriptor.address.port_id();
            external_slots[slot_index] = Some(self.id_generator.generate_external_id());
        }
    }

    /// Returns the `ConnectionId` to use for a new edge leaving `start_port_address`.
    ///
    /// If the port already has an entry in `port_address_to_connection_id_map`, this is a
    /// fan-out: the existing `ConnectionId` (and its pool buffer) is reused so all downstream
    /// consumers share the same audio data.
    ///
    /// If the port has no entry yet, a fresh buffer is allocated, the source node's output
    /// slot is updated, and the connection is recorded in `graph_items`.
    fn resolve_connection_for_output_port(
        &mut self,
        start_port_address: PortAddress,
        end_port_address: PortAddress,
    ) -> Result<ConnectionId, GraphConnectionError> {
        if let Some(&existing_connection_id) = self
            .port_address_to_connection_id_map
            .get(&start_port_address)
        {
            // Fan-out: the source port already has a buffer — reuse it for the new consumer.
            return Ok(existing_connection_id);
        }

        // First connection from this port: allocate a buffer and record the source slot.
        let connection = Connection::new(self, start_port_address, end_port_address);
        let connection_id = connection.connection_id;

        let channels = self
            .port_address_to_channel_count
            .get(&start_port_address)
            .copied()
            .unwrap_or(1);
        self.allocate_empty_buffer_for_connection(connection_id, channels)?;
        self.port_address_to_connection_id_map
            .insert(start_port_address, connection_id);

        let start_node_id = start_port_address.node_id();
        if let Some(port_map) = self.node_connection_id_map.get_mut(&start_node_id)
            && let Some(slot) = port_map
                .output_port_slots
                .get_mut(**start_port_address.port_id())
        {
            *slot = Some(connection_id);
        }

        self.graph_items
            .insert(*connection_id, GraphItem::Connection(connection));

        Ok(connection_id)
    }

    /// Extracts raw buffer pointers for a single direction (inputs or outputs) of one node.
    ///
    /// Each slot in `connection_ids` maps to one port. For ports backed by pool buffers,
    /// this extracts the `UnsafeCell`-derived pointer (SRW provenance) along with the channel
    /// count. External ports are not in the slot arrays (their entries are `None`), so they
    /// naturally produce `None` here and are patched from `CompiledStep::external_*_slots`
    /// during each `run` call.
    ///
    /// SAFETY: All returned pointers are derived from `UnsafeCell::get()` on pool buffers,
    /// giving them SRW (SharedReadWrite) provenance. See `run()` for the full safety argument.
    unsafe fn resolve_audio_buffer_pointers(
        buffer_pool: &crate::primitives::BufferPool,
        connection_ids: &[Option<ConnectionId>],
    ) -> Box<[Option<RawAudioBuffer>]> {
        connection_ids
            .iter()
            .map(|connection_id_opt| {
                let connection_id = connection_id_opt.as_ref()?;
                let cb: &ChannelledBuffer = buffer_pool.get(connection_id)?;
                // SAFETY: UnsafeCell::get() yields *mut [Sample] with SRW
                // (SharedReadWrite) provenance, which lives at the base of the
                // Stacked Borrows borrow stack and is never invalidated by Unique
                // retags from mutable accesses in other nodes' process() calls.
                let raw_ptr: *mut [Sample] = cb.data.get();
                Some(RawAudioBuffer {
                    ptr: unsafe { NonNull::new_unchecked(raw_ptr) },
                    channels: cb.channels,
                })
            })
            .collect()
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
            .map(|node_index_vec: Vec<petgraph::graph::NodeIndex>| {
                node_index_vec
                    .into_iter()
                    .map(|node_index| *self.petgraph_index_to_node_id.get(&node_index).unwrap())
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
    fn traverse_graph<F>(&self, sccs: &[Vec<Id>], visited_set: &mut HashSet<Id>, callback: &mut F)
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
            self.visit_node(*leaf_node_id, sccs, visited_set, callback);
        }

        // Collect all nodes that were not reachable from any leaf node.
        // These are, by definition, part of cycles.
        let mut unvisited_node_ids: Vec<Id> = self
            .graph_items
            .iter()
            .filter_map(|(node_id, graph_item)| {
                if let GraphItem::Node(_node) = graph_item
                    && visited_set.get(node_id).is_none()
                {
                    return Some(*node_id);
                }
                None
            })
            .collect();

        // Sort cyclical nodes so the lowest-priority one acts as a stand-in leaf node,
        // giving the cycle a deterministic entry point.
        unvisited_node_ids.sort_by(|id_a, id_b| {
            compare_nodes_by_priority(self.get_node(id_a).unwrap(), self.get_node(id_b).unwrap())
        });
        unvisited_node_ids.reverse();

        for unvisited_node_id in unvisited_node_ids {
            if visited_set.get(&unvisited_node_id).is_some() {
                continue;
            }
            visited_set.insert(unvisited_node_id);
            self.visit_node(unvisited_node_id, sccs, visited_set, callback);
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
        callback: &mut F,
    ) where
        F: FnMut(Id),
    {
        let petgraph_index = self.node_id_to_petgraph_index.get(&current_id).unwrap();
        let mut neighbor_ids: Vec<Id> = self
            .graph
            // Traverse backwards/upwards from leaf nodes toward source nodes.
            .neighbors_directed(*petgraph_index, petgraph::Direction::Incoming)
            .map(|neighbor_petgraph_index| {
                *self
                    .petgraph_index_to_node_id
                    .get(&neighbor_petgraph_index)
                    .unwrap()
            })
            // Exclude self-loops — they are handled by the SCC detection.
            .filter(|&neighbor_id| neighbor_id != current_id)
            .collect();

        neighbor_ids.sort_by(|id_a, id_b| {
            compare_nodes_by_priority(self.get_node(id_a).unwrap(), self.get_node(id_b).unwrap())
        });

        // Partition neighbors in one pass: cycles are visited first to isolate their
        // run order before the acyclic portion of the graph is processed.
        let (cyclical_neighbor_ids, acyclical_neighbor_ids): (Vec<Id>, Vec<Id>) = neighbor_ids
            .into_iter()
            .partition(|&neighbor_id| self.is_cyclical_node(neighbor_id, sccs));

        for cyclical_neighbor_id in cyclical_neighbor_ids {
            if visited_set.get(&cyclical_neighbor_id).is_some() {
                continue;
            }
            visited_set.insert(cyclical_neighbor_id);
            self.visit_node(cyclical_neighbor_id, sccs, visited_set, callback);
        }

        for acyclical_neighbor_id in acyclical_neighbor_ids {
            if visited_set.get(&acyclical_neighbor_id).is_some() {
                continue;
            }
            visited_set.insert(acyclical_neighbor_id);
            self.visit_node(acyclical_neighbor_id, sccs, visited_set, callback);
        }

        // Visit the current node last (post-order).
        callback(current_id);
    }

    /// A node is cyclical if it belongs to an SCC of length > 1 or has a direct self-loop.
    fn is_cyclical_node(&self, id: Id, sccs: &[Vec<Id>]) -> bool {
        let scc = sccs
            .iter()
            .find(|scc: &&Vec<Id>| scc.iter().any(|scc_id| *id == **scc_id))
            .unwrap();

        if scc.len() > 1 {
            return true;
        }

        let petgraph_index = self.node_id_to_petgraph_index.get(&id).unwrap();
        let neighbor_ids: Vec<Id> = self
            .graph
            .neighbors_directed(*petgraph_index, petgraph::Direction::Incoming)
            .map(|neighbor_petgraph_index| {
                *self
                    .petgraph_index_to_node_id
                    .get(&neighbor_petgraph_index)
                    .unwrap()
            })
            .collect();

        neighbor_ids.contains(&id)
    }

    /// Returns the number of port slots needed to hold all ports in the given groups.
    /// With dense port IDs this equals the actual port count; with sparse IDs it overallocates.
    fn count_ports(port_descriptor_groups: &[Option<&[PortDescriptor]>]) -> usize {
        port_descriptor_groups
            .iter()
            .filter_map(|group| *group)
            .flat_map(|descriptors: &[PortDescriptor]| descriptors.iter())
            .map(|descriptor| **descriptor.address.port_id())
            .max()
            .map(|max_id| max_id + 1)
            .unwrap_or(0)
    }

    /// Checks that all port IDs across the given groups form a dense 0..n sequence.
    /// Called separately for the input group and the output group during `add()`.
    fn validate_dense_port_ids(
        port_descriptor_groups: &[Option<&[PortDescriptor]>],
    ) -> Result<(), GraphAddError> {
        let mut port_ids: Vec<usize> = port_descriptor_groups
            .iter()
            .filter_map(|group| *group)
            .flat_map(|descriptors: &[PortDescriptor]| descriptors.iter())
            .map(|descriptor| **descriptor.address.port_id())
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
                let node_ptr: *mut dyn ErasedAudioNode = {
                    let Some(GraphItem::Node(Node::AudioNode(node_box))) =
                        self.graph_items.get_mut(&id)
                    else {
                        return None;
                    };
                    // SAFETY: Box heap allocation is stable; moving the Box (e.g. on IntMap
                    // rehash) does not move the heap data the Box points to.
                    &mut **node_box as *mut dyn ErasedAudioNode
                };

                let connection_id_map = self.node_connection_id_map.get(&NodeId::from(id))?;

                // SAFETY: see run() for the full provenance and aliasing argument.
                let input_buffer_ptrs = unsafe {
                    Self::resolve_audio_buffer_pointers(
                        &self.buffer_pool,
                        &connection_id_map.input_port_slots,
                    )
                };
                let output_buffer_ptrs = unsafe {
                    Self::resolve_audio_buffer_pointers(
                        &self.buffer_pool,
                        &connection_id_map.output_port_slots,
                    )
                };

                Some(CompiledStep {
                    node: node_ptr,
                    input_buffer_ptrs,
                    output_buffer_ptrs,
                    external_output_slots: connection_id_map.external_output_slots.clone(),
                    external_input_slots: connection_id_map.external_input_slots.clone(),
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
        channels: usize,
    ) -> Result<(), BufferAlreadyAllocated> {
        if self.buffer_pool.contains_key(&connection_id) {
            return Err(BufferAlreadyAllocated);
        }

        let total_samples = *self.block_size * channels;
        let mut buf: Vec<Sample> = Vec::with_capacity(total_samples);
        buf.resize(total_samples, Sample::default());

        // SAFETY: `UnsafeCell<[Sample]>` is `#[repr(transparent)]` over `[Sample]`,
        // so `Box<[Sample]>` and `Box<UnsafeCell<[Sample]>>` have identical layouts.
        let cell_box: Box<UnsafeCell<[Sample]>> = unsafe {
            alloc::boxed::Box::from_raw(
                alloc::boxed::Box::into_raw(buf.into_boxed_slice()) as *mut UnsafeCell<[Sample]>
            )
        };

        self.buffer_pool.insert(
            connection_id,
            ChannelledBuffer {
                channels,
                data: cell_box,
            },
        );

        Ok(())
    }
}

impl GenerateId for Graph {
    fn generate_id(&mut self) -> Id {
        self.id_generator().generate_id()
    }
}

impl crate::traits::Graph for Graph {
    fn add_audio_node<P: DescribePorts, N: AudioNode + GetPortDescriptors<P> + 'static>(
        &mut self,
        node: N,
    ) -> Result<NodeHandle<P>, GraphAddError> {
        // TODO: check that the adding the node is valid before making it
        // - node id should not already be in the graph
        // - node id should not be weirdly higher than the rest

        let port_descriptors: P = node.get_port_descriptors();

        // Enforce dense port IDs so that slice lengths equal actual port counts.
        // All inputs share one PortId namespace; all outputs share another.
        Self::validate_dense_port_ids(&[port_descriptors.input_ports()])?;
        Self::validate_dense_port_ids(&[port_descriptors.output_ports()])?;

        let node_id = NodeId::from(node.node_id());

        // Pre-allocate slot arrays indexed by PortId.
        let num_input_ports = Self::count_ports(&[port_descriptors.input_ports()]);
        let num_output_ports = Self::count_ports(&[port_descriptors.output_ports()]);

        // None for now: updated at `connect` time. Indexed by PortId.
        let input_port_slots: Vec<Option<ConnectionId>> = vec![None; num_input_ports];
        let output_port_slots: Vec<Option<ConnectionId>> = vec![None; num_output_ports];
        let mut external_input_slots: Vec<Option<ExternalConnectionId>> =
            vec![None; num_input_ports];
        let mut external_output_slots: Vec<Option<ExternalConnectionId>> =
            vec![None; num_output_ports];

        // External ports are provided by the caller at run time; assign ExternalConnectionIds now.
        // `register_external_port_addresses` inspects each descriptor's direction and only fills
        // slots for ExternalInput/ExternalOutput ports.
        self.register_external_port_addresses(
            port_descriptors.output_ports().unwrap_or(&[]),
            &mut external_output_slots,
        );
        self.register_external_port_addresses(
            port_descriptors.input_ports().unwrap_or(&[]),
            &mut external_input_slots,
        );

        // Register channel counts for all port addresses so that `connect()` can validate
        // matching channel counts and `resolve_connection_for_output_port` can allocate
        // correctly-sized pool buffers.
        for descriptors in [
            port_descriptors.input_ports(),
            port_descriptors.output_ports(),
        ]
        .into_iter()
        .flatten()
        {
            for descriptor in descriptors {
                self.port_address_to_channel_count
                    .insert(descriptor.address, descriptor.channels);
            }
        }

        self.node_connection_id_map.insert(
            node_id,
            NodeConnectionIdMap {
                input_port_slots: input_port_slots.into_boxed_slice(),
                output_port_slots: output_port_slots.into_boxed_slice(),
                external_input_slots: external_input_slots.into_boxed_slice(),
                external_output_slots: external_output_slots.into_boxed_slice(),
            },
        );

        let node_handle = NodeHandle::new(
            node_id,
            port_descriptors,
            self.node_connection_id_map[&node_id]
                .external_input_slots
                .clone(),
            self.node_connection_id_map[&node_id]
                .external_output_slots
                .clone(),
        );

        self.graph_items.insert(
            *node_id,
            GraphItem::Node(Node::AudioNode(Box::new(node) as Box<dyn ErasedAudioNode>)),
        );

        self.register_node_in_petgraph(node_id);

        // Every new node starts as a leaf; it loses this status when it gains an outgoing edge.
        self.leaf_nodes.insert(node_id);

        self.invalidate_compiled_plan();

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

        // Validate that both ports carry the same number of channels.
        let start_channels = self
            .port_address_to_channel_count
            .get(&start_port_address)
            .copied()
            .unwrap_or(1);
        let end_channels = self
            .port_address_to_channel_count
            .get(&end_port_address)
            .copied()
            .unwrap_or(1);
        if start_channels != end_channels {
            return Err(GraphConnectionError::ChannelCountMismatch {
                start_channels,
                end_channels,
            });
        }

        let connection_id =
            self.resolve_connection_for_output_port(start_port_address, end_port_address)?;

        // The destination node's input slot always gets updated, even for fan-out connections.
        self.port_address_to_connection_id_map
            .insert(end_port_address, connection_id);

        let end_node_id = end_port_address.node_id();
        if let Some(port_map) = self.node_connection_id_map.get_mut(&end_node_id)
            && let Some(slot) = port_map
                .input_port_slots
                .get_mut(**end_port_address.port_id())
        {
            *slot = Some(connection_id);
        }

        let start_node_id = start_port_address.node_id();
        let start_index = *self.node_id_to_petgraph_index.get(&*start_node_id).unwrap();
        let end_index = *self.node_id_to_petgraph_index.get(&*end_node_id).unwrap();
        self.graph.add_edge(start_index, end_index, connection_id);

        // A node with an outgoing edge is no longer a leaf.
        self.leaf_nodes.remove(&start_node_id);

        self.invalidate_compiled_plan();

        Ok(self)
    }

    fn run<A: crate::traits::AudioBuffer, M: crate::traits::AudioBufferMut>(
        &mut self,
        inputs: &[Option<A>],
        outputs: &mut [Option<M>],
        current_time: CurrentTime,
    ) -> Result<(), GraphRunError> {
        self.ensure_compiled_plan();

        let compiled_plan = self.compiled_plan.as_mut().unwrap();

        for step in compiled_plan.iter_mut() {
            // Patch output slots whose buffers are supplied by the caller for this block.
            for (slot, external_connection_id_opt) in step.external_output_slots.iter().enumerate()
            {
                let Some(external_connection_id) = external_connection_id_opt else {
                    continue;
                };
                step.output_buffer_ptrs[slot] =
                    // the generic argument is not guaranteed to be in the
                    // memory layout we need, so we do a quick conversion here
                    outputs.get_mut(***external_connection_id).and_then(|opt| opt.as_mut()).map(RawAudioBuffer::from);
            }

            // Patch input slots whose buffers are supplied by the caller for this block.
            for (slot, external_connection_id_opt) in step.external_input_slots.iter().enumerate() {
                let Some(external_connection_id) = external_connection_id_opt else {
                    continue;
                };
                step.input_buffer_ptrs[slot] =
                    // the generic argument is not guaranteed to be in the
                    // memory layout we need, so we do a quick conversion here
                    inputs.get(***external_connection_id).and_then(|opt| opt.as_ref()).map(RawAudioBuffer::from);
            }

            // SAFETY:
            // 1. Buffer addresses are stable: `allocate_empty_buffer_for_connection` is only
            //    called from `connect()`, never during `run()`, so no heap allocation moves
            //    while `compiled_plan` is in use.  External input/output pointers come from
            //    the caller's AudioBuffer/AudioBufferMut, which are valid for the duration
            //    of `run()`.
            // 2. No mutable aliasing on outputs: each output slot has a unique `ConnectionId` or
            //    `ExternalConnectionId`. No two `*mut [Sample]` pointers in `output_ptrs` alias the same memory.
            // 3. Visit order enforces exclusive access: by the time a node reads a buffer as
            //    input, the upstream node that writes it has already completed its `process` call.
            // 4. Stacked Borrows / pointer provenance: all pool buffer pointers are derived from
            //    `UnsafeCell::get()` (see `resolve_audio_buffer_pointers`), giving them SRW
            //    provenance that is never invalidated by the Unique retags created inside
            //    `process()` calls.
            //    see: https://github.com/rust-lang/unsafe-code-guidelines/blob/master/wip/stacked-borrows.md
            // 5. repr(C) layout: RawAudioBuffer and AudioBuffer<'_>/AudioBufferMut<'_> have
            //    the same first two fields (ptr: NonNull<[Sample]>, channels: usize) with
            //    PhantomData being zero-sized and last. Option<RawAudioBuffer> uses the same
            //    null-pointer niche as Option<AudioBuffer<'_>>. The transmute is valid.
            let input_buffers: &[Option<AudioBuffer<'_>>] =
                unsafe { transmute(step.input_buffer_ptrs.as_ref()) };
            let output_buffers: &mut [Option<AudioBufferMut<'_>>] =
                unsafe { transmute(step.output_buffer_ptrs.as_mut()) };

            // SAFETY: `step.node` points into the heap allocation of a `Box<dyn AudioNode>`
            // stored in `self.graph_items`. Moving the `Box` (e.g. on IntMap rehash) does not
            // move the heap data, so the pointer remains valid. No topology modification
            // (add/connect/disconnect/remove) can occur concurrently with `run()`.
            unsafe {
                (&mut *step.node).process(
                    input_buffers,
                    output_buffers,
                    step.block_size,
                    current_time,
                )
            }
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
                let multiply_node_handle_1 = graph.add_audio_node(multiply_node_1).unwrap();
                let multiply_node_handle_2 = graph.add_audio_node(multiply_node_2).unwrap();
                let constant_node_handle_1 = graph.add_audio_node(constant_node_1).unwrap();
                let constant_node_handle_2 = graph.add_audio_node(constant_node_2).unwrap();

                let visit_order = graph.compute_new_visit_order();
                // Unconnected nodes have no topological constraint; they are ordered by
                // priority (= node_id), which reflects creation order (lower ID = created first).
                // constant_1 (id=0) and constant_2 (id=1) were created before
                // multiply_1 (id=2) and multiply_2 (id=3).
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

                let constant_node_handle = graph.add_audio_node(constant_node).unwrap();
                let multiply_node_handle = graph.add_audio_node(multiply_node).unwrap();

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

                let node_0_handle = graph.add_audio_node(node_0).unwrap();
                let node_1_handle = graph.add_audio_node(node_1).unwrap();
                let node_2_handle = graph.add_audio_node(node_2).unwrap();
                let node_3_handle = graph.add_audio_node(node_3).unwrap();
                let node_4_handle = graph.add_audio_node(node_4).unwrap();

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

                let node_0_handle = graph.add_audio_node(node_0).unwrap();
                let node_1_handle = graph.add_audio_node(node_1).unwrap();
                let node_2_handle = graph.add_audio_node(node_2).unwrap();
                let node_3_handle = graph.add_audio_node(node_3).unwrap();
                let node_4_handle = graph.add_audio_node(node_4).unwrap();

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
                implementations::{ConstantNode, Graph, MultiplyNode, OutputNode},
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
                let constant_node_1_handle = graph.add_audio_node(constant_node_1).unwrap();

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

                let constant_node_1_handle = graph.add_audio_node(constant_node_1).unwrap();
                let constant_node_2_handle = graph.add_audio_node(constant_node_2).unwrap();

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

                let constant_node_0_handle = graph.add_audio_node(constant_node_0).unwrap();
                let constant_node_1_handle = graph.add_audio_node(constant_node_1).unwrap();
                let constant_node_2_handle = graph.add_audio_node(constant_node_2).unwrap();

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

                let constant_node_0_handle = graph.add_audio_node(constant_node_0).unwrap();
                let constant_node_1_handle = graph.add_audio_node(constant_node_1).unwrap();

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

                let constant_node_0_handle = graph.add_audio_node(constant_node_0).unwrap();
                let constant_node_1_handle = graph.add_audio_node(constant_node_1).unwrap();

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

                let node_0_handle = graph.add_audio_node(node_0).unwrap();
                let node_1_handle = graph.add_audio_node(node_1).unwrap();
                let node_2_handle = graph.add_audio_node(node_2).unwrap();
                let node_3_handle = graph.add_audio_node(node_3).unwrap();
                let node_4_handle = graph.add_audio_node(node_4).unwrap();
                let node_5_handle = graph.add_audio_node(node_5).unwrap();
                let node_6_handle = graph.add_audio_node(node_6).unwrap();
                let node_7_handle = graph.add_audio_node(node_7).unwrap();

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
                let node_2_handle = graph.add_audio_node(node_2).unwrap();
                let node_6_handle = graph.add_audio_node(node_6).unwrap();
                let node_4_handle = graph.add_audio_node(node_4).unwrap();
                let node_5_handle = graph.add_audio_node(node_5).unwrap();
                let node_3_handle = graph.add_audio_node(node_3).unwrap();
                let node_0_handle = graph.add_audio_node(node_0).unwrap();
                let node_7_handle = graph.add_audio_node(node_7).unwrap();
                let node_1_handle = graph.add_audio_node(node_1).unwrap();

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
        use crate::primitives::Sample;
        use alloc::vec::Vec;

        /// Converts a slice of `f32` literals into `Vec<Sample>` for concise assertions.
        fn samples(values: &[f32]) -> Vec<Sample> {
            values.iter().map(|&v| Sample::from(v)).collect()
        }

        /// Returns the single external output `ExternalConnectionId` from an `OutputNode` handle.
        macro_rules! ext_output_id {
            ($handle:expr) => {
                $handle.external_output_connection_ids()
                    [**OutputNodePortDescriptors::EXTERNAL_OUTPUT_PORT_ID]
                    .unwrap()
            };
        }

        mod buffer_processing_results {

            use crate::{
                implementations::{
                    AudioBuffer, AudioBufferMut, ConstantNode, Graph, MultiplyNode, OutputNode,
                    OutputNodePortDescriptors, graph::graph_tests::audio_processing::samples,
                },
                primitives::{CurrentTime, Sample},
                test_utils::{OutputBufferKeyMapping, outputs_from_buffer_mapping},
                traits::{AudioBuffer as _, Graph as GraphTrait},
            };
            use alloc::vec::Vec;

            #[test]
            fn only_output_node() {
                let mut graph = Graph::new();
                let output_node = OutputNode::new(&mut graph);
                let output_node_handle = graph.add_audio_node(output_node).unwrap();
                let external_output_connection_id = output_node_handle
                    .external_output_connection_ids()
                    [**OutputNodePortDescriptors::EXTERNAL_OUTPUT_PORT_ID]
                    .unwrap();

                let mut output_buffer = vec![Sample::default()];
                let num_channels = 1;
                let mut output_mapping = [OutputBufferKeyMapping {
                    id: external_output_connection_id,
                    buffer: Some(output_buffer.as_mut_slice()),
                    num_channels,
                }];
                let mut outputs = outputs_from_buffer_mapping(&mut output_mapping);

                graph
                    .run::<AudioBuffer<'_>, _>(&[], &mut outputs, CurrentTime::from(0.0))
                    .unwrap();

                assert_eq!(
                    outputs[**external_output_connection_id]
                        .as_ref()
                        .unwrap()
                        .mono()
                        .unwrap(),
                    &[Sample::default()]
                );
            }

            #[test]
            fn no_external_output_supplied_none_written() {
                let mut graph = Graph::new();

                let constant_node = ConstantNode::new(&mut graph);
                let output_node = OutputNode::new(&mut graph);

                let constant_node_handle = graph.add_audio_node(constant_node).unwrap();
                let output_node_handle = graph.add_audio_node(output_node).unwrap();

                graph
                    .connect(
                        constant_node_handle.output_port_address(),
                        output_node_handle.input_port_address(),
                    )
                    .unwrap();

                let mut outputs: Vec<Option<AudioBufferMut<'_>>> = vec![];

                graph
                    .run::<AudioBuffer<'_>, _>(&[], &mut outputs, CurrentTime::default())
                    .unwrap();

                assert!(outputs.is_empty());
            }

            #[test]
            fn constant_node_with_value_to_output_node() {
                let mut graph = Graph::new();

                let expected_sample_value = 5i32;
                let constant_node =
                    ConstantNode::new_with_value(&mut graph, Sample::from(expected_sample_value));
                let output_node = OutputNode::new(&mut graph);

                let constant_node_handle = graph.add_audio_node(constant_node).unwrap();
                let output_node_handle = graph.add_audio_node(output_node).unwrap();
                let external_output_connection_id = output_node_handle
                    .external_output_connection_ids()
                    [**OutputNodePortDescriptors::EXTERNAL_OUTPUT_PORT_ID]
                    .unwrap();

                graph
                    .connect(
                        constant_node_handle.output_port_address(),
                        output_node_handle.input_port_address(),
                    )
                    .unwrap();

                let mut output_buffer = vec![Sample::default()];
                let num_channels = 1;
                let mut output_mapping = [OutputBufferKeyMapping {
                    id: external_output_connection_id,
                    buffer: Some(output_buffer.as_mut_slice()),
                    num_channels,
                }];
                let mut outputs = outputs_from_buffer_mapping(&mut output_mapping);

                graph
                    .run::<AudioBuffer<'_>, _>(&[], &mut outputs, CurrentTime::default())
                    .unwrap();

                assert_eq!(
                    outputs[**external_output_connection_id]
                        .as_ref()
                        .unwrap()
                        .mono()
                        .unwrap(),
                    &[Sample::from(expected_sample_value)]
                );
            }

            #[test]
            fn empty_graph_run_succeeds() {
                let mut graph = Graph::new();
                let mut outputs: Vec<Option<AudioBufferMut<'_>>> = vec![];
                assert!(
                    graph
                        .run::<AudioBuffer<'_>, _>(&[], &mut outputs, CurrentTime::default())
                        .is_ok()
                );
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

                let const_2_handle = graph.add_audio_node(const_2).unwrap();
                let const_3_handle = graph.add_audio_node(const_3).unwrap();
                let multiply_handle = graph.add_audio_node(multiply).unwrap();
                let output_handle = graph.add_audio_node(output).unwrap();

                let external_output_connection_id = output_handle.external_output_connection_ids()
                    [**OutputNodePortDescriptors::EXTERNAL_OUTPUT_PORT_ID]
                    .unwrap();

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

                let mut output_buffer = vec![Sample::default()];
                let num_channels = 1;
                let mut output_mapping = [OutputBufferKeyMapping {
                    id: external_output_connection_id,
                    buffer: Some(output_buffer.as_mut_slice()),
                    num_channels,
                }];
                let mut outputs = outputs_from_buffer_mapping(&mut output_mapping);

                graph
                    .run::<AudioBuffer<'_>, _>(&[], &mut outputs, CurrentTime::default())
                    .unwrap();

                assert_eq!(
                    outputs[**external_output_connection_id]
                        .as_ref()
                        .unwrap()
                        .mono()
                        .unwrap(),
                    samples(&[6.0]).as_slice()
                );
            }

            #[test]
            fn fan_out_one_constant_to_multiple_outputs() {
                // const(5.0) ─── output_1
                //            └── output_2
                //            └── output_3
                let mut graph = Graph::new();
                let constant = ConstantNode::new_with_value(&mut graph, 5.0f32);
                let output_1 = OutputNode::new(&mut graph);
                let output_2 = OutputNode::new(&mut graph);
                let output_3 = OutputNode::new(&mut graph);

                let constant_handle = graph.add_audio_node(constant).unwrap();
                let output_1_handle = graph.add_audio_node(output_1).unwrap();
                let output_2_handle = graph.add_audio_node(output_2).unwrap();
                let output_3_handle = graph.add_audio_node(output_3).unwrap();

                let external_connection_id_1 = output_1_handle.external_output_connection_ids()
                    [**OutputNodePortDescriptors::EXTERNAL_OUTPUT_PORT_ID]
                    .unwrap();
                let external_connection_id_2 = output_2_handle.external_output_connection_ids()
                    [**OutputNodePortDescriptors::EXTERNAL_OUTPUT_PORT_ID]
                    .unwrap();
                let external_connection_id_3 = output_3_handle.external_output_connection_ids()
                    [**OutputNodePortDescriptors::EXTERNAL_OUTPUT_PORT_ID]
                    .unwrap();

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
                    .unwrap()
                    .connect(
                        constant_handle.output_port_address(),
                        output_3_handle.input_port_address(),
                    )
                    .unwrap();

                let mut out_buf_1 = vec![Sample::default()];
                let mut out_buf_2 = vec![Sample::default()];
                let mut out_buf_3 = vec![Sample::default()];

                let mut outputs: Vec<Option<AudioBufferMut<'_>>> = Vec::from([
                    Some(AudioBufferMut::new(&mut out_buf_1, 1).unwrap()),
                    Some(AudioBufferMut::new(&mut out_buf_2, 1).unwrap()),
                    Some(AudioBufferMut::new(&mut out_buf_3, 1).unwrap()),
                ]);

                graph
                    .run::<AudioBuffer<'_>, _>(&[], &mut outputs, CurrentTime::default())
                    .unwrap();

                assert_eq!(
                    outputs[**external_connection_id_1]
                        .as_ref()
                        .unwrap()
                        .mono()
                        .unwrap(),
                    samples(&[5.0]).as_slice()
                );
                assert_eq!(
                    outputs[**external_connection_id_2]
                        .as_ref()
                        .unwrap()
                        .mono()
                        .unwrap(),
                    samples(&[5.0]).as_slice()
                );
                assert_eq!(
                    outputs[**external_connection_id_3]
                        .as_ref()
                        .unwrap()
                        .mono()
                        .unwrap(),
                    samples(&[5.0]).as_slice()
                );
            }

            #[test]
            fn larger_block_size_fills_all_samples() {
                let block_size = 4usize;
                let mut graph = Graph::with_block_size(block_size);
                let constant = ConstantNode::new_with_value(&mut graph, 9.0f32);
                let output = OutputNode::new(&mut graph);

                let constant_handle = graph.add_audio_node(constant).unwrap();
                let output_handle = graph.add_audio_node(output).unwrap();
                let external_output_connection_id = output_handle.external_output_connection_ids()
                    [**OutputNodePortDescriptors::EXTERNAL_OUTPUT_PORT_ID]
                    .unwrap();

                graph
                    .connect(
                        constant_handle.output_port_address(),
                        output_handle.input_port_address(),
                    )
                    .unwrap();

                let mut output_buffer = vec![Sample::default(); block_size];
                let num_channels = 1;
                let mut output_mapping = [OutputBufferKeyMapping {
                    id: external_output_connection_id,
                    buffer: Some(output_buffer.as_mut_slice()),
                    num_channels,
                }];
                let mut outputs = outputs_from_buffer_mapping(&mut output_mapping);

                graph
                    .run::<AudioBuffer<'_>, _>(&[], &mut outputs, CurrentTime::default())
                    .unwrap();

                assert_eq!(
                    outputs[**external_output_connection_id]
                        .as_ref()
                        .unwrap()
                        .mono()
                        .unwrap(),
                    samples(&[9.0, 9.0, 9.0, 9.0]).as_slice()
                );
            }

            #[test]
            fn multiple_run_calls_produce_consistent_results() {
                let mut graph = Graph::new();
                let constant = ConstantNode::new_with_value(&mut graph, 3.0f32);
                let output = OutputNode::new(&mut graph);

                let constant_handle = graph.add_audio_node(constant).unwrap();
                let output_handle = graph.add_audio_node(output).unwrap();
                let external_output_connection_id = output_handle.external_output_connection_ids()
                    [**OutputNodePortDescriptors::EXTERNAL_OUTPUT_PORT_ID]
                    .unwrap();

                graph
                    .connect(
                        constant_handle.output_port_address(),
                        output_handle.input_port_address(),
                    )
                    .unwrap();

                for _ in 0..3 {
                    let mut output_buffer = vec![Sample::default()];
                    let num_channels = 1;
                    let mut output_mapping = [OutputBufferKeyMapping {
                        id: external_output_connection_id,
                        buffer: Some(output_buffer.as_mut_slice()),
                        num_channels,
                    }];
                    let mut outputs = outputs_from_buffer_mapping(&mut output_mapping);

                    graph
                        .run::<AudioBuffer<'_>, _>(&[], &mut outputs, CurrentTime::default())
                        .unwrap();

                    assert_eq!(
                        outputs[**external_output_connection_id]
                            .as_ref()
                            .unwrap()
                            .mono()
                            .unwrap(),
                        samples(&[3.0]).as_slice()
                    );
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

                let const_2_handle = graph.add_audio_node(const_2).unwrap();
                let const_3_handle = graph.add_audio_node(const_3).unwrap();
                let const_4_handle = graph.add_audio_node(const_4).unwrap();
                let multiply_1_handle = graph.add_audio_node(multiply_1).unwrap();
                let multiply_2_handle = graph.add_audio_node(multiply_2).unwrap();
                let output_handle = graph.add_audio_node(output).unwrap();

                let external_connection_id = output_handle.external_output_connection_ids()
                    [**OutputNodePortDescriptors::EXTERNAL_OUTPUT_PORT_ID]
                    .unwrap();

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

                let mut out_buf = vec![Sample::default()];
                let mut outputs = vec![Some(AudioBufferMut::new(&mut out_buf, 1).unwrap())];

                graph
                    .run::<AudioBuffer<'_>, _>(&[], &mut outputs, CurrentTime::default())
                    .unwrap();

                assert_eq!(
                    outputs[**external_connection_id]
                        .as_ref()
                        .unwrap()
                        .mono()
                        .unwrap(),
                    samples(&[24.0]).as_slice()
                );
            }

            #[test]
            fn multiply_node_with_unconnected_inputs_outputs_zero() {
                // A multiply node with no connections defaults to 0.0 * 0.0 = 0.0
                let mut graph = Graph::new();
                let multiply = MultiplyNode::new(&mut graph);
                let output = OutputNode::new(&mut graph);

                let multiply_handle = graph.add_audio_node(multiply).unwrap();
                let output_handle = graph.add_audio_node(output).unwrap();
                let external_connection_id = ext_output_id!(output_handle);

                graph
                    .connect(
                        multiply_handle.output_port_address(),
                        output_handle.input_port_address(),
                    )
                    .unwrap();

                let mut out_buf = vec![Sample::default()];
                let mut outputs = vec![Some(AudioBufferMut::new(&mut out_buf, 1).unwrap())];

                graph
                    .run::<AudioBuffer<'_>, _>(&[], &mut outputs, CurrentTime::default())
                    .unwrap();

                assert_eq!(
                    outputs[**external_connection_id]
                        .as_ref()
                        .unwrap()
                        .mono()
                        .unwrap(),
                    samples(&[0.0]).as_slice()
                );
            }
        }

        mod channel_count_validation {
            use crate::errors::AudioNodeRunError;
            use crate::primitives::{BlockSize, CurrentTime, Id, Priority};
            use crate::traits::Graph as GraphTrait;
            use crate::{
                errors::GraphConnectionError,
                implementations::{ConstantNode, Graph, OutputNode},
                primitives::{NodeId, PortAddress, PortAddressDirection, PortDescriptor, PortId},
                traits::{
                    AudioNode, DescribePorts, GenerateId, GetNodeId, GetPortDescriptors,
                    GetPriority,
                },
            };
            use core::ops::Deref;

            // A mono-output node (channels = 1)
            struct MonoOutputNode {
                node_id: NodeId,
                descriptors: MonoOutputDescriptors,
            }

            #[derive(Copy, Clone)]
            struct MonoOutputDescriptors {
                output: [PortDescriptor; 1],
            }

            impl MonoOutputNode {
                fn new<G: GenerateId>(id_gen: &mut G) -> Self {
                    let node_id = NodeId::from(id_gen.generate_id());
                    Self {
                        node_id,
                        descriptors: MonoOutputDescriptors {
                            output: [PortDescriptor {
                                address: PortAddress::new(
                                    node_id,
                                    PortId::new(0),
                                    PortAddressDirection::Output,
                                ),
                                channels: 1,
                            }],
                        },
                    }
                }
            }

            impl GetNodeId for MonoOutputNode {
                fn node_id(&self) -> Id {
                    *self.node_id
                }
            }
            impl GetPriority for MonoOutputNode {
                fn get_priority(&self) -> Priority {
                    (**self.node_id).into()
                }
            }
            impl MonoOutputDescriptors {
                fn output_port_address(&self) -> PortAddress {
                    self.output[0].address
                }
            }
            impl DescribePorts for MonoOutputDescriptors {
                fn output_ports(&self) -> Option<&[PortDescriptor]> {
                    Some(&self.output)
                }
            }
            impl Deref for MonoOutputNode {
                type Target = MonoOutputDescriptors;
                fn deref(&self) -> &Self::Target {
                    &self.descriptors
                }
            }
            impl GetPortDescriptors<MonoOutputDescriptors> for MonoOutputNode {
                fn get_port_descriptors(&self) -> MonoOutputDescriptors {
                    self.descriptors
                }
            }
            impl AudioNode for MonoOutputNode {
                fn process<A: crate::traits::AudioBuffer, M: crate::traits::AudioBufferMut>(
                    &mut self,
                    _inputs: &[Option<A>],
                    _outputs: &mut [Option<M>],
                    _block_size: BlockSize,
                    _current_time: CurrentTime,
                ) -> Result<(), AudioNodeRunError> {
                    Ok(())
                }
            }

            // A stereo-input node (channels = 2)
            struct StereoInputNode {
                node_id: NodeId,
                descriptors: StereoInputDescriptors,
            }

            #[derive(Copy, Clone)]
            struct StereoInputDescriptors {
                input: [PortDescriptor; 1],
            }

            impl StereoInputNode {
                fn new<G: GenerateId>(id_gen: &mut G) -> Self {
                    let node_id = NodeId::from(id_gen.generate_id());
                    Self {
                        node_id,
                        descriptors: StereoInputDescriptors {
                            input: [PortDescriptor {
                                address: PortAddress::new(
                                    node_id,
                                    PortId::new(0),
                                    PortAddressDirection::Input,
                                ),
                                channels: 2,
                            }],
                        },
                    }
                }
            }

            impl GetNodeId for StereoInputNode {
                fn node_id(&self) -> Id {
                    *self.node_id
                }
            }
            impl GetPriority for StereoInputNode {
                fn get_priority(&self) -> Priority {
                    (**self.node_id).into()
                }
            }
            impl StereoInputDescriptors {
                fn input_port_address(&self) -> PortAddress {
                    self.input[0].address
                }
            }
            impl DescribePorts for StereoInputDescriptors {
                fn input_ports(&self) -> Option<&[PortDescriptor]> {
                    Some(&self.input)
                }
            }
            impl Deref for StereoInputNode {
                type Target = StereoInputDescriptors;
                fn deref(&self) -> &Self::Target {
                    &self.descriptors
                }
            }
            impl GetPortDescriptors<StereoInputDescriptors> for StereoInputNode {
                fn get_port_descriptors(&self) -> StereoInputDescriptors {
                    self.descriptors
                }
            }
            impl AudioNode for StereoInputNode {
                fn process<A: crate::traits::AudioBuffer, M: crate::traits::AudioBufferMut>(
                    &mut self,
                    _inputs: &[Option<A>],
                    _outputs: &mut [Option<M>],
                    _block_size: BlockSize,
                    _current_time: CurrentTime,
                ) -> Result<(), AudioNodeRunError> {
                    Ok(())
                }
            }

            #[test]
            fn mismatched_channel_counts_return_error() {
                let mut graph = Graph::new();
                let mono_out = MonoOutputNode::new(&mut graph);
                let stereo_in = StereoInputNode::new(&mut graph);

                let mono_handle = graph.add_audio_node(mono_out).unwrap();
                let stereo_handle = graph.add_audio_node(stereo_in).unwrap();

                let result = graph.connect(
                    mono_handle.output_port_address(),
                    stereo_handle.input_port_address(),
                );

                assert!(matches!(
                    result,
                    Err(GraphConnectionError::ChannelCountMismatch {
                        start_channels: 1,
                        end_channels: 2,
                    })
                ));
            }

            #[test]
            fn matching_channel_counts_succeed() {
                let mut graph = Graph::new();
                let constant = ConstantNode::new_with_value(&mut graph, 1.0f32);
                let output = OutputNode::new(&mut graph);

                let constant_handle = graph.add_audio_node(constant).unwrap();
                let output_handle = graph.add_audio_node(output).unwrap();

                // Both are mono (channels = 1), so this should succeed.
                assert!(
                    graph
                        .connect(
                            constant_handle.output_port_address(),
                            output_handle.input_port_address(),
                        )
                        .is_ok()
                );
            }
        }

        mod external_inputs {
            use core::ops::Deref;

            use crate::primitives::CurrentTime;
            use crate::test_utils::{
                InputBufferKeyMapping, OutputBufferKeyMapping, inputs_from_buffer_mapping,
                outputs_from_buffer_mapping,
            };
            use crate::traits::Graph as GraphTrait;
            use crate::{
                errors::AudioNodeRunError,
                implementations::Graph,
                primitives::{
                    BlockSize, Id, NodeId, PortAddress, PortAddressDirection, PortDescriptor,
                    PortId, Priority, Sample,
                },
                traits::{
                    AudioNode, DescribePorts, GenerateId, GetNodeId, GetPortDescriptors,
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
                fn new<G: GenerateId>(id_gen: &mut G) -> Self {
                    let node_id = NodeId::from(id_gen.generate_id());
                    Self {
                        node_id,
                        port_descriptors: PassthroughPortDescriptors::new(node_id),
                    }
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
                fn process<A: crate::traits::AudioBuffer, M: crate::traits::AudioBufferMut>(
                    &mut self,
                    inputs: &[Option<A>],
                    outputs: &mut [Option<M>],
                    _block_size: BlockSize,
                    _current_time: CurrentTime,
                ) -> Result<(), AudioNodeRunError> {
                    let Some(out_buf) = outputs.get_mut(0).and_then(|o| o.as_mut()) else {
                        return Ok(());
                    };
                    let input_block: &[Sample] = inputs
                        .first()
                        .and_then(|o| o.as_ref())
                        .map(|b| b.mono())
                        .transpose()?
                        .unwrap_or(&[]);
                    let output_block = out_buf.mono_mut()?;
                    for (o, &i) in output_block.iter_mut().zip(input_block.iter()) {
                        *o = i;
                    }
                    Ok(())
                }
            }

            #[derive(Copy, Clone)]
            struct PassthroughPortDescriptors {
                external_input: [PortDescriptor; 1],
                external_output: [PortDescriptor; 1],
            }

            impl PassthroughPortDescriptors {
                const EXTERNAL_INPUT_PORT_ID: PortId = PortId::new(0);
                const EXTERNAL_OUTPUT_PORT_ID: PortId = PortId::new(0);

                fn new(node_id: NodeId) -> Self {
                    Self {
                        external_input: [PortDescriptor {
                            address: PortAddress::new(
                                node_id,
                                Self::EXTERNAL_INPUT_PORT_ID,
                                PortAddressDirection::ExternalInput,
                            ),
                            channels: 1,
                        }],
                        external_output: [PortDescriptor {
                            address: PortAddress::new(
                                node_id,
                                Self::EXTERNAL_OUTPUT_PORT_ID,
                                PortAddressDirection::ExternalOutput,
                            ),
                            channels: 1,
                        }],
                    }
                }
            }

            impl DescribePorts for PassthroughPortDescriptors {
                fn input_ports(&self) -> Option<&[PortDescriptor]> {
                    Some(&self.external_input)
                }

                fn output_ports(&self) -> Option<&[PortDescriptor]> {
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
                let handle = graph.add_audio_node(node).unwrap();

                let external_input_connection_id = handle.external_input_connection_ids()
                    [**PassthroughPortDescriptors::EXTERNAL_INPUT_PORT_ID]
                    .unwrap();
                let external_output_connection_id = handle.external_output_connection_ids()
                    [**PassthroughPortDescriptors::EXTERNAL_OUTPUT_PORT_ID]
                    .unwrap();

                let input_buffer = [
                    Sample::from(1.0f32),
                    Sample::from(2.0f32),
                    Sample::from(3.0f32),
                    Sample::from(4.0f32),
                ];

                let num_channels = 1;

                let inputs = inputs_from_buffer_mapping(&[(InputBufferKeyMapping {
                    id: external_input_connection_id,
                    buffer: input_buffer.as_slice(),
                    num_channels,
                })]);

                let mut output_buffer = vec![Sample::default(); block_size];
                let mut output_mapping = [OutputBufferKeyMapping {
                    id: external_output_connection_id,
                    buffer: Some(output_buffer.as_mut_slice()),
                    num_channels,
                }];
                let mut outputs = outputs_from_buffer_mapping(&mut output_mapping);

                graph
                    .run(&inputs, &mut outputs, CurrentTime::default())
                    .unwrap();

                assert_eq!(output_buffer, input_buffer.as_slice());
            }
        }
    }
}
