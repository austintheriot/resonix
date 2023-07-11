use std::{
    cell::RefCell,
    collections::{HashSet, VecDeque},
    ops::{Deref, DerefMut},
};

use nohash_hasher::{IntMap, IntSet};
use petgraph::{
    stable_graph::{EdgeIndex, NodeIndex},
    visit::Dfs,
    Direction, Graph,
};

#[cfg(feature = "dac")]
use {resonix_dac::DACConfig, std::sync::Arc};

use crate::{
    messages::{AddNodeError, ConnectError, UpdateNodeError, UpdateNodeMessage},
    BoxedNode, Connection, DACNode, Node, NodeType, NodeUid,
};
use resonix_core::NumChannels;

/// Cloning the audio context is an outward clone of the
/// audio context handle
#[derive(Debug, Default, Clone)]
pub struct Processor {
    graph: Graph<RefCell<BoxedNode>, RefCell<Connection>>,
    /// Maps node uids to their `NodeIndex` in the graph.
    /// This value must be updated if/when removing nodes
    /// from the graph is supported
    node_uid_to_node_index_map: IntMap<u32, NodeIndex>,
    /// Makes sure we don't try to add duplicate nodes to the graph
    node_uids: IntSet<u32>,
    /// Created while nodes are being added, so that when it's time `run` the graph,
    /// there is no analysis that needs to happen, it's just simple list of node_indexes,
    /// ordered by what value they need to be visited in
    visit_order: Option<Vec<NodeIndex>>,
    /// All the input nodes in the audio graph--these must be visited first
    /// for maximum processing efficiency
    input_node_indexes: Vec<NodeIndex>,
    /// All the DAC nodes in the audio graph--these should be easily
    /// retrieved for copying audio graph information into the
    /// audio thread output.
    dac_node_indexes: Vec<NodeIndex>,
    /// These are all the nodes that should get updated with audio information
    /// (such as `sample_rate`, `num_channels`, etc.) whenever the DAC is initialized
    audio_update_node_indexes: Vec<NodeIndex>,
    incoming_connection_indexes: IntMap<u32, Vec<EdgeIndex>>,
    outgoing_connection_indexes: IntMap<u32, Vec<EdgeIndex>>,
    uid_counter: u32,
}

impl Processor {
    pub fn new() -> Self {
        Default::default()
    }

    /// Executes the audio graph
    #[inline]
    pub(crate) fn run(&mut self) {
        if self.visit_order.is_none() {
            self.initialize_visit_order();
        }

        for node_index in self.visit_order.as_ref().unwrap() {
            let node = &self.graph[*node_index];
            let node_uid = node.borrow().uid();
            let incoming_edge_indexes = self.incoming_connection_indexes(&node_uid).unwrap_or(&[]);
            let outgoing_edge_indexes = self.outgoing_connection_indexes(&node_uid).unwrap_or(&[]);

            let mut incoming_connections = {
                incoming_edge_indexes
                    .iter()
                    .map(|i| self.graph.edge_weight(*i).unwrap().borrow())
            };

            let mut outgoing_connections = {
                outgoing_edge_indexes
                    .iter()
                    .map(|i| self.graph.edge_weight(*i).unwrap().borrow_mut())
            };

            node.borrow_mut()
                .process(&mut incoming_connections, &mut outgoing_connections)
        }
    }

    /// This pre-processes the audio graph to a create a fixed graph traversal order
    /// such that nodes are only visited once all their connections are guaranteed
    /// to have been initialized by their parent nodes (if applicable)
    ///
    /// It is called by default on the first run of the audio graph, but can be called
    /// manually before then to preemptively prepare the audio processor before it's time
    /// to start processing audio in real time.
    ///
    /// Subsequent calls to this function are ignored.
    pub fn initialize_visit_order(&mut self) {
        if self.visit_order.is_some() {
            return;
        }

        // if there are no inputs to the graph, then there is nothing to traverse
        if self.input_node_indexes.is_empty() {
            self.visit_order = Some(Vec::new());
            return;
        }

        // allows shuffling nodes around while determining a path through the graph
        let mut in_progress_visit_order = VecDeque::with_capacity(self.graph.node_count());
        // the final order that will be used to traverse the graph when calling `run`
        let mut final_visit_order = Vec::with_capacity(self.graph.node_count());
        // keeps track of which connections have been visited from a parent node--
        // this mimics the behavior of nodes in a true `run`, where outgoing connections
        // are initialized by parent nodes
        let mut connection_visit_set: IntSet<u32> = IntSet::default();

        // prevents cycling endlessly through graph
        const MAX_GRAPH_VISITS: u32 = 65536;
        let mut graph_visits = 0;

        {
            // keep track of which nodes have been added to the in_progress_visit_order vec
            let mut node_set: HashSet<NodeIndex> = HashSet::new();

            // initialize visit order with all nodes, starting with the inputs
            for input_index in &self.input_node_indexes {
                if !node_set.contains(input_index) {
                    in_progress_visit_order.push_back(*input_index);
                    node_set.insert(*input_index);
                }

                let mut dfs = Dfs::new(&self.graph, *input_index);
                while let Some(node_index) = dfs.next(&self.graph) {
                    if !node_set.contains(&node_index) {
                        in_progress_visit_order.push_back(node_index);
                        node_set.insert(node_index);
                    }
                }
            }
        }

        // find a valid path through graph, such that all inputs
        // are initialized for each node before that node's `process` function is run
        while let Some(node_index) = in_progress_visit_order.pop_front() {
            graph_visits += 1;

            // todo: make this a configurable number and a recoverable error
            if graph_visits > MAX_GRAPH_VISITS {
                panic!(
                    r#"Too many iterations reached while searching for an allowable signal path though audio graph. 
                This probably indicates a bug in your audio graph, such as an unintended infinite loop."#
                );
            }

            let mut incoming_connections = self
                .graph
                .edges_directed(node_index, Direction::Incoming)
                .map(|edge_reference| edge_reference.weight());

            // skip for now if inputs have not been initialized
            if incoming_connections.any(|incoming_connection| {
                !connection_visit_set.contains(incoming_connection.borrow().uid())
            }) {
                in_progress_visit_order.push_back(node_index);
                continue;
            }

            // if made it this far, we know that this node is valid to visit
            // at this point in the graph traversal, since the all the node's
            // inputs were visited prior to calling this node's `process` function
            final_visit_order.push(node_index);

            let outgoing_connections = self.graph.edges_directed(node_index, Direction::Outgoing);

            // mark all outgoing connections from this node has having been visited
            outgoing_connections.for_each(|edge_reference| {
                connection_visit_set.insert(*edge_reference.weight().borrow().uid());
            });
        }

        self.visit_order = Some(final_visit_order);
    }

    /// Used in audio thread to extract audio information from all the DACs
    #[cfg(feature = "dac")]
    pub(crate) fn dac_nodes_sum(&self, num_channels: NumChannels) -> Vec<f32> {
        let mut multi_channel_sum = vec![0.0; *num_channels];
        self.dac_node_indexes.iter().for_each(|i: &NodeIndex| {
            let dac_node_ref = &self.graph[*i].borrow();
            let dac_data = dac_node_ref
                .as_any()
                .downcast_ref::<DACNode>()
                .unwrap()
                .data();

            // TODO - currently, this ignoring any size mismatches between num channels for the DAC
            // and the number of actual audio channels going out. Update this to upmix / downmix as necessary
            multi_channel_sum
                .iter_mut()
                .zip(dac_data.iter())
                .for_each(|(output, channel)| {
                    *output += channel;
                });
        });

        multi_channel_sum
    }

    fn connect_with_indexes(
        &mut self,
        parent_node_uid: NodeUid,
        child_node_uid: NodeUid,
        from_index: usize,
        to_index: usize,
    ) -> Result<EdgeIndex, ConnectError> {
        let parent_node_index = self
            .node_uid_to_node_index_map
            .get(&parent_node_uid)
            .ok_or(ConnectError::NodeUidNotFound {
                node_uid: parent_node_uid,
            })?;
        let child_node_index = self.node_uid_to_node_index_map.get(&child_node_uid).ok_or(
            ConnectError::NodeUidNotFound {
                node_uid: child_node_uid,
            },
        )?;

        // check if connection indexes are out of bounds
        let (parent_uuid, child_uuid, num_channels) = {
            let parent_node =
                &self
                    .graph
                    .node_weight(*parent_node_index)
                    .ok_or(ConnectError::NodeNotFound {
                        node_index: *parent_node_index,
                    })?;
            let child_node =
                &self
                    .graph
                    .node_weight(*child_node_index)
                    .ok_or(ConnectError::NodeNotFound {
                        node_index: *child_node_index,
                    })?;

            Self::check_connection_index_out_of_bounds(
                &parent_node.borrow(),
                &child_node.borrow(),
                from_index,
                to_index,
            )?;

            Self::check_num_channels_compatibility(&parent_node.borrow(), &child_node.borrow())?;

            if parent_node_uid == child_node_uid {
                return Err(ConnectError::GraphCycleFound {
                    parent_node_name: parent_node.borrow().name(),
                    child_node_name: child_node.borrow().name(),
                });
            }

            self.check_for_cyclical_connection(
                child_node_index,
                parent_node_index,
                &parent_node.borrow(),
                &child_node.borrow(),
            )?;

            (
                parent_node.borrow().uid(),
                child_node.borrow().uid(),
                parent_node.borrow().num_outgoing_channels(),
            )
        };

        // add connection to graph
        let edge_index = self.graph.add_edge(
            *parent_node_index,
            *child_node_index,
            RefCell::new(Connection::from_indexes(num_channels, from_index, to_index)),
        );

        self.add_outgoing_connection_index(parent_uuid, edge_index);
        self.add_incoming_connection_index(child_uuid, edge_index);

        Ok(edge_index)
    }

    fn incoming_connection_indexes(&self, uid: &u32) -> Option<&[EdgeIndex]> {
        self.incoming_connection_indexes
            .get(uid)
            .map(|indexes| indexes.as_slice())
    }

    fn outgoing_connection_indexes(&self, uid: &u32) -> Option<&[EdgeIndex]> {
        self.outgoing_connection_indexes
            .get(uid)
            .map(|indexes| indexes.as_slice())
    }

    fn add_incoming_connection_index(&mut self, uid: NodeUid, edge_index: EdgeIndex) {
        self.incoming_connection_indexes
            .entry(uid)
            .and_modify(|edge_indexes| edge_indexes.push(edge_index))
            .or_insert_with(|| vec![edge_index]);
    }

    fn add_outgoing_connection_index(&mut self, uid: NodeUid, edge_index: EdgeIndex) {
        self.outgoing_connection_indexes
            .entry(uid)
            .and_modify(|edge_indexes| edge_indexes.push(edge_index))
            .or_insert_with(|| vec![edge_index]);
    }

    fn check_connection_index_out_of_bounds(
        parent_node: &BoxedNode,
        child_node: &BoxedNode,
        from_index: usize,
        to_index: usize,
    ) -> Result<(), ConnectError> {
        if from_index >= parent_node.num_output_connections()
            || to_index >= child_node.num_input_connections()
        {
            return Err(ConnectError::IncorrectIndex {
                expected_from_index: parent_node.num_input_connections() - 1,
                expected_to_index: child_node.num_input_connections() - 1,
                from_index,
                to_index,
                parent_node_name: parent_node.name(),
                child_node_name: child_node.name(),
            });
        }

        Ok(())
    }

    fn check_for_cyclical_connection(
        &self,
        child_node_index: &NodeIndex,
        parent_node_index: &NodeIndex,
        parent_node: &BoxedNode,
        child_node: &BoxedNode,
    ) -> Result<(), ConnectError> {
        let cycle_found = 'block: {
            let starting_node_index = child_node_index;
            let ending_node_index = parent_node_index;
            let mut dfs = Dfs::new(&self.graph, *starting_node_index);
            while let Some(current_node_index) = dfs.next(&self.graph) {
                if &current_node_index == ending_node_index {
                    break 'block true;
                }
            }

            false
        };

        if cycle_found {
            return Err(ConnectError::GraphCycleFound {
                parent_node_name: parent_node.name(),
                child_node_name: child_node.name(),
            });
        }

        Ok(())
    }

    /// Regardless of the node type, the outgoing number of channels of a parent
    /// should always match the incoming number of channels for the child node
    ///
    /// Note: given a single node, the number its incoming and outgoing connections
    /// do not necessarily need to match. For example, a Downmix node could take multichannel
    /// input and mix it down to single-channel output.
    fn check_num_channels_compatibility(
        parent_node: &BoxedNode,
        child_node: &BoxedNode,
    ) -> Result<(), ConnectError> {
        let parent_node_num_outgoing_channels = parent_node.num_outgoing_channels();
        let child_node_num_incoming_channels = child_node.num_incoming_channels();
        if parent_node_num_outgoing_channels != child_node_num_incoming_channels {
            let parent_node_name = parent_node.name();
            let child_node_name = child_node.name();
            return Err(ConnectError::IncompatibleNumChannels {
                parent_node_num_outgoing_channels,
                child_node_num_incoming_channels,
                parent_node_name,
                child_node_name,
            });
        }

        Ok(())
    }

    pub fn reset_visit_order_cache(&mut self) {
        self.visit_order.take();
    }

    pub fn add_node<N: Node + 'static>(&mut self, mut node: N) -> Result<NodeUid, AddNodeError> {
        if node.uid() != 0 {
            return Err(AddNodeError::NodeAlreadyAssociatedToContext { name: node.name() });
        }

        node.set_uid(self.next_uid());

        let uid = node.uid();

        // make node immutable for rest of code block
        let node = node;

        if self.node_uids.contains(&uid) {
            return Err(AddNodeError::AlreadyExists { name: node.name() });
        }

        let is_input = node.node_type() == NodeType::Input;

        let is_dac = { node.as_any().downcast_ref::<DACNode>().is_some() };

        #[cfg(feature = "dac")]
        let requires_audio_updates = node.requires_audio_updates();

        let node_index = self.graph.add_node(RefCell::new(Box::new(node)));

        self.node_uid_to_node_index_map.insert(uid, node_index);

        if is_input {
            self.input_node_indexes.push(node_index);
        }

        if is_dac {
            self.dac_node_indexes.push(node_index);
        }

        #[cfg(feature = "dac")]
        if requires_audio_updates {
            self.audio_update_node_indexes.push(node_index);
        }

        self.reset_visit_order_cache();

        Ok(uid)
    }

    /// Updates internal data of nodes to match any audio data
    /// from the environment (sample rate, num output channels, etc.)
    #[cfg(feature = "dac")]
    pub fn update_audio_nodes(&mut self, dac_config: Arc<DACConfig>) {
        self.audio_update_node_indexes
            .iter()
            .filter_map(|i| self.graph.node_weight(*i))
            .for_each(|node| {
                node.borrow_mut()
                    .update_from_dac_config(Arc::clone(&dac_config));
            });
    }

    pub fn connect(
        &mut self,
        parent_node_uid: NodeUid,
        child_node_uid: NodeUid,
    ) -> Result<EdgeIndex, ConnectError> {
        let result = self.connect_with_indexes(
            parent_node_uid,
            child_node_uid,
            Default::default(),
            Default::default(),
        );

        // reset visit order cache
        if result.is_ok() {
            self.reset_visit_order_cache();
        }

        result
    }

    fn next_uid(&mut self) -> u32 {
        let value = self.uid_counter;
        self.uid_counter += 1;
        value
    }

    fn boxed_node_by_uid(&self, uid: &u32) -> Option<&RefCell<BoxedNode>> {
        self.node_uid_to_node_index_map
            .get(uid)
            .and_then(|node_index| self.graph.node_weight(*node_index))
    }

    #[cfg(feature = "dac")]
    pub(crate) fn handle_update_node_message(
        &mut self,
        update_node_message: UpdateNodeMessage,
    ) -> Result<(), UpdateNodeError> {
        let UpdateNodeMessage { node_uid, .. } = update_node_message;
        let boxed_node = self
            .boxed_node_by_uid(&node_uid)
            .ok_or(UpdateNodeError::NodeNotFound { uid: node_uid })?;
        boxed_node
            .borrow_mut()
            .handle_update_node_message(update_node_message)?;

        Ok(())
    }
}

impl Deref for Processor {
    type Target = Graph<RefCell<BoxedNode>, RefCell<Connection>>;

    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}

impl DerefMut for Processor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.graph
    }
}

#[cfg(test)]
mod test_processor {

    use crate::{
        messages::ConnectError, ConstantNode, DACNode, PassThroughNode, Processor, SineNode,
    };

    #[test]
    fn rejects_connection_to_self() {
        let mut processor = Processor::default();

        let pass_through_node = PassThroughNode::new(1);

        let uid = processor.add_node(pass_through_node).unwrap();

        let result = processor.connect(uid, uid);

        assert!(result.is_err());
    }

    #[test]
    fn rejects_single_edge_cyclical_connection() {
        let mut processor = Processor::default();

        let pass_through_node_a: PassThroughNode = PassThroughNode::new(1);
        let pass_through_node_b: PassThroughNode = PassThroughNode::new(1);

        let pass_through_node_a_uid = processor.add_node(pass_through_node_a).unwrap();
        let pass_through_node_b_uid = processor.add_node(pass_through_node_b).unwrap();

        processor
            .connect(pass_through_node_a_uid, pass_through_node_b_uid)
            .unwrap();
        let result = processor.connect(pass_through_node_b_uid, pass_through_node_a_uid);

        assert!(matches!(result, Err(ConnectError::GraphCycleFound { .. })));
    }

    #[test]
    fn rejects_multi_edge_cyclical_connection() {
        let mut processor = Processor::default();

        let pass_through_node_a: PassThroughNode = PassThroughNode::new(1);
        let pass_through_node_b: PassThroughNode = PassThroughNode::new(1);
        let pass_through_node_c: PassThroughNode = PassThroughNode::new(1);
        let pass_through_node_d: PassThroughNode = PassThroughNode::new(1);

        let pass_through_node_a_uid = processor.add_node(pass_through_node_a).unwrap();
        let pass_through_node_b_uid = processor.add_node(pass_through_node_b).unwrap();
        let pass_through_node_c_uid = processor.add_node(pass_through_node_c).unwrap();
        let pass_through_node_d_uid = processor.add_node(pass_through_node_d).unwrap();

        processor
            .connect(pass_through_node_a_uid, pass_through_node_b_uid)
            .unwrap();
        processor
            .connect(pass_through_node_b_uid, pass_through_node_c_uid)
            .unwrap();
        processor
            .connect(pass_through_node_c_uid, pass_through_node_d_uid)
            .unwrap();
        let result = processor.connect(pass_through_node_d_uid, pass_through_node_a_uid);

        assert!(matches!(result, Err(ConnectError::GraphCycleFound { .. })));
    }

    #[test]
    fn allows_retrieving_boxed_node_by_uid() {
        let mut processor = Processor::default();

        let sine_node = SineNode::new(1, 440.0);

        let uid = processor.add_node(sine_node).unwrap();

        assert!(processor.boxed_node_by_uid(&uid).is_some());
    }

    #[test]
    fn allows_retrieving_node_by_uid() {
        let mut processor = Processor::default();

        let sine_node = SineNode::new(1, 440.0);

        let uid = processor.add_node(sine_node).unwrap();
        let boxed_node = processor.boxed_node_by_uid(&uid).unwrap();
        let boxed_node_ref = boxed_node.borrow();
        let sine_node = boxed_node_ref.as_any().downcast_ref::<SineNode>();

        assert!(sine_node.is_some());
    }

    #[test]
    fn running_processor_should_fill_connections_with_data() {
        let mut processor = Processor::default();
        let constant_node = ConstantNode::new(1, 0.5);
        let pass_through_node = PassThroughNode::new(1);
        let dac_node = DACNode::new(1);

        let constant_node_uid = processor.add_node(constant_node).unwrap();
        let pass_through_node_uid = processor.add_node(pass_through_node).unwrap();
        let dac_node_uid = processor.add_node(dac_node).unwrap();

        let constant_to_pass_through_edge_index = processor
            .connect(constant_node_uid, pass_through_node_uid)
            .unwrap();
        let pass_through_to_dac_edge_index = processor
            .connect(pass_through_node_uid, dac_node_uid)
            .unwrap();

        // no data yet in connections
        {
            let constant_to_pass_through_edge = processor
                .graph
                .edge_weight(constant_to_pass_through_edge_index)
                .unwrap();
            let pass_through_to_dac_edge = processor
                .graph
                .edge_weight(pass_through_to_dac_edge_index)
                .unwrap();
            assert_eq!(constant_to_pass_through_edge.borrow().data(), &vec![0.0]);
            assert_eq!(pass_through_to_dac_edge.borrow().data(), &vec![0.0]);
        }

        processor.run();

        // data is in connections now
        {
            let constant_to_pass_through_edge = processor
                .graph
                .edge_weight(constant_to_pass_through_edge_index)
                .unwrap();
            let pass_through_to_dac_edge = processor
                .graph
                .edge_weight(pass_through_to_dac_edge_index)
                .unwrap();
            assert_eq!(constant_to_pass_through_edge.borrow().data(), &vec![0.5]);
            assert_eq!(pass_through_to_dac_edge.borrow().data(), &vec![0.5]);
        }
    }
}
