use std::{
    collections::{HashMap, HashSet, VecDeque},
    ops::{Deref, DerefMut},
    ptr::{addr_of, addr_of_mut},
};

use petgraph::{
    data::DataMap,
    stable_graph::{EdgeIndex, NodeIndex},
    visit::Dfs,
    Direction, Graph,
};

use uuid::Uuid;

use crate::{AddConnectionError, BoxedNode, Connection, DACNode, Node, NodeType};

#[derive(thiserror::Error, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConnectError {
    #[error("Node could not be found in the audio graph for index {node_index:?}. Are you sure you added it?")]
    NodeNotFound { node_index: NodeIndex },
    #[error("Node connection from {parent_node_name:?} to {child_node_name:?} failed. Expected `from_index` to be a max of {expected_from_index:?} and `to_index`  to be a max of {expected_to_index:?}. Received `from_index`  of {from_index:?} and `to_index` of {to_index:?}")]
    IncorrectIndex {
        expected_from_index: usize,
        expected_to_index: usize,
        from_index: usize,
        to_index: usize,
        parent_node_name: String,
        child_node_name: String,
    },
    #[error("Node connection failed. Original error: {0:?}")]
    AddConnectionError(#[from] AddConnectionError),
    #[error("A message was sent to the `Processor` in the audio thread to connect 2 nodes, but no corresponding response was received")]
    NoMatchingMessageReceived,
}

#[derive(thiserror::Error, Debug)]
pub enum AddNodeError {
    #[error("Cannot add {name:?} to the audio graph, since it has already been added.")]
    AlreadyExists { name: String },
    #[error("A message was sent to the `Processor` in the audio thread to add a node, but no corresponding response was received")]
    NoMatchingMessageReceived,
}

/// Cloning the audio context is an outward clone of the
/// audio context handle
#[derive(Debug, Default, Clone)]
pub struct Processor {
    graph: Graph<Uuid, Uuid>,
    nodes_by_id: HashMap<Uuid, BoxedNode>,
    connections_by_id: HashMap<Uuid, Connection>,
    node_uuids: HashSet<Uuid>,
    visit_order: Option<Vec<NodeIndex>>,
    input_node_indexes: Vec<NodeIndex>,
    dac_node_indexes: Vec<NodeIndex>,
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
            // it is safe to immutably borrow `node_mut` for its connections, AS LONG AS the
            // connections are not touched within the Node's `process` implementation
            // todo => refactor `Node::process` to make this impossible (and therefore actually safe)

            let node_uuid = &self.graph[*node_index];

            let (incoming_connection_uuids, outgoing_connection_uuids) = {
                let node = self.nodes_by_id.get(node_uuid).unwrap();
                let incoming_edge_indexes = node.incoming_connection_indexes();
                let outgoing_edge_indexes = node.outgoing_connection_indexes();

                let incoming_connection_uuids = incoming_edge_indexes
                    .iter()
                    .map(|i| *self.graph.edge_weight(*i).unwrap());

                let outgoing_connection_uuids = outgoing_edge_indexes
                    .iter()
                    .map(|i| *self.graph.edge_weight(*i).unwrap());

                (incoming_connection_uuids, outgoing_connection_uuids)
            };

            let mut incoming_connections = Vec::new();
            for uuid in incoming_connection_uuids {
                incoming_connections.push(self.connections_by_id.remove(&uuid).unwrap());
            }

            let mut outgoing_connections = Vec::new();
            for uuid in outgoing_connection_uuids {
                outgoing_connections.push(self.connections_by_id.remove(&uuid).unwrap());
            }

            let node_mut = self.nodes_by_id.get_mut(node_uuid).unwrap();
            node_mut.process(
                &mut incoming_connections.iter(),
                &mut outgoing_connections.iter_mut(),
            );

            self.connections_by_id.extend(
                incoming_connections
                    .into_iter()
                    .map(|connection| (*connection.uuid(), connection)),
            );
            self.connections_by_id.extend(
                outgoing_connections
                    .into_iter()
                    .map(|connection| (*connection.uuid(), connection)),
            );
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
        let mut connection_visit_set: HashSet<Uuid> = HashSet::new();

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
            if incoming_connections
                .any(|incoming_connection| !connection_visit_set.contains(incoming_connection))
            {
                in_progress_visit_order.push_back(node_index);
                continue;
            }

            // if made it this far, we know that this node is valid to visit
            // at this point in the graph traversal, since the all the node's
            // inputs were visited prior to calling this node's `process` function
            final_visit_order.push(node_index);

            let outgoing_connection = self.graph.edges_directed(node_index, Direction::Outgoing);

            // mark all outgoing connections from this node has having been visited
            outgoing_connection.for_each(|edge_reference| {
                connection_visit_set.insert(*edge_reference.weight());
            });
        }

        self.visit_order = Some(final_visit_order);
    }

    /// Used in audio thread to extract audio information from all the DACs
    #[cfg(feature = "dac")]
    pub(crate) fn dac_nodes_sum(&self) -> f32 {
        self.dac_node_indexes
            .iter()
            .map(|i: &NodeIndex| {
                let uuid = &self.graph[*i];
                self.nodes_by_id[uuid]
                    .as_any()
                    .downcast_ref::<DACNode>()
                    .unwrap()
                    .data()
            })
            .sum()
    }

    fn connect_with_indexes(
        &mut self,
        parent_node_index: NodeIndex,
        child_node_index: NodeIndex,
        from_index: usize,
        to_index: usize,
    ) -> Result<EdgeIndex, ConnectError> {
        // check if connection indexes are out of bounds
        {
            let parent_node = &self
                .graph
                .node_weight(parent_node_index)
                .map(|uuid| self.nodes_by_id.get(uuid).unwrap())
                .ok_or(ConnectError::NodeNotFound {
                    node_index: parent_node_index,
                })?;
            let child_node = &self
                .graph
                .node_weight(child_node_index)
                .map(|uuid| self.nodes_by_id.get(uuid).unwrap())
                .ok_or(ConnectError::NodeNotFound {
                    node_index: child_node_index,
                })?;

            Self::check_index_out_of_bounds(parent_node, child_node, from_index, to_index)?;
        }

        let connection = Connection::from_indexes(from_index, to_index);
        let connection_uuid = *connection.uuid();

        self.connections_by_id.insert(connection_uuid, connection);

        // add connection to graph
        let edge_index = self
            .graph
            .add_edge(parent_node_index, child_node_index, connection_uuid);

        // add connection indexes to nodes themselves for faster retrieval later
        self.graph
            .node_weight_mut(parent_node_index)
            .map(|uuid| self.nodes_by_id.get_mut(uuid).unwrap())
            .ok_or(ConnectError::NodeNotFound {
                node_index: parent_node_index,
            })?
            .add_outgoing_connection_index(edge_index)
            .unwrap();

        self.graph
            .node_weight_mut(child_node_index)
            .map(|uuid| self.nodes_by_id.get_mut(uuid).unwrap())
            .ok_or(ConnectError::NodeNotFound {
                node_index: child_node_index,
            })?
            .add_incoming_connection_index(edge_index)
            .unwrap();

        Ok(edge_index)
    }

    fn check_index_out_of_bounds(
        parent_node: &BoxedNode,
        child_node: &BoxedNode,
        from_index: usize,
        to_index: usize,
    ) -> Result<(), ConnectError> {
        if from_index >= parent_node.num_outputs() || to_index >= child_node.num_inputs() {
            return Err(ConnectError::IncorrectIndex {
                expected_from_index: parent_node.num_inputs() - 1,
                expected_to_index: child_node.num_inputs() - 1,
                from_index,
                to_index,
                parent_node_name: parent_node.name(),
                child_node_name: child_node.name(),
            });
        }

        Ok(())
    }

    pub fn reset_visit_order_cache(&mut self) {
        self.visit_order.take();
    }

    pub fn add_node<N: Node + 'static>(&mut self, node: N) -> Result<NodeIndex, AddNodeError> {
        if self.node_uuids.contains(node.uuid()) {
            return Err(AddNodeError::AlreadyExists { name: node.name() });
        }

        let is_input = node.node_type() == NodeType::Input;

        let is_dac = { node.as_any().downcast_ref::<DACNode>().is_some() };

        let node_uuid = *node.uuid();

        self.nodes_by_id.insert(node_uuid, Box::new(node));

        let node_index = self.graph.add_node(node_uuid);

        if is_input {
            self.input_node_indexes.push(node_index);
        }

        if is_dac {
            self.dac_node_indexes.push(node_index);
        }

        self.reset_visit_order_cache();

        Ok(node_index)
    }

    pub fn connect(
        &mut self,
        parent_node_index: NodeIndex,
        child_node_index: NodeIndex,
    ) -> Result<EdgeIndex, ConnectError> {
        let result = self.connect_with_indexes(
            parent_node_index,
            child_node_index,
            Default::default(),
            Default::default(),
        );

        // reset visit order cache
        if result.is_ok() {
            self.reset_visit_order_cache();
        }

        result
    }

    pub(crate) fn node_by_node_index<N: Node + 'static>(&self, node_index: NodeIndex) -> Option<&N> {
        self.boxed_node_by_node_index(node_index)
            .and_then(|boxed_node| (*boxed_node).as_any().downcast_ref::<N>())
    }

    pub(crate) fn boxed_node_by_node_index(&self, node_index: NodeIndex) -> Option<&BoxedNode> {
        self.graph
            .node_weight(node_index)
            .and_then(|uuid| self.nodes_by_id.get(uuid))
    }

    pub(crate) fn boxed_node_mut_by_node_index(&mut self, node_index: NodeIndex) -> Option<&mut BoxedNode> {
        self.graph
            .node_weight_mut(node_index)
            .and_then(|uuid| self.nodes_by_id.get_mut(uuid))
    }

    pub(crate) fn connection_by_edge_index(&self, edge_index: EdgeIndex) -> Option<&Connection> {
        self.graph
            .edge_weight(edge_index)
            .and_then(|uuid| self.connections_by_id.get(uuid))
    }
}

impl Deref for Processor {
    type Target = Graph<Uuid, Uuid>;

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
    use crate::{ConstantNode, DACNode, PassThroughNode, Processor};

    #[test]
    fn running_processor_should_fill_connections_with_data() {
        let mut processor = Processor::default();
        let constant_node = ConstantNode::new_with_signal_value(0.5);
        let pass_through_node = PassThroughNode::new();
        let dac_node = DACNode::new();

        let constant_node_index = processor.add_node(constant_node).unwrap();
        let pass_through_node_index = processor.add_node(pass_through_node).unwrap();
        let dac_node_index = processor.add_node(dac_node).unwrap();

        let constant_to_pass_through_edge_index = processor
            .connect(constant_node_index, pass_through_node_index)
            .unwrap();
        let pass_through_to_dac_edge_index = processor
            .connect(pass_through_node_index, dac_node_index)
            .unwrap();

        // no data yet in connections
        {
            let constant_to_pass_through_edge = processor
                .connection_by_edge_index(constant_to_pass_through_edge_index)
                .unwrap();
            let pass_through_to_dac_edge = processor
                .connection_by_edge_index(pass_through_to_dac_edge_index)
                .unwrap();
            assert_eq!(constant_to_pass_through_edge.data(), 0.0);
            assert_eq!(pass_through_to_dac_edge.data(), 0.0);
        }

        processor.run();

        // data is in connections now
        {
            let constant_to_pass_through_edge = processor
                .connection_by_edge_index(constant_to_pass_through_edge_index)
                .unwrap();
            let pass_through_to_dac_edge = processor
                .connection_by_edge_index(pass_through_to_dac_edge_index)
                .unwrap();
            assert_eq!(constant_to_pass_through_edge.data(), 0.5);
            assert_eq!(pass_through_to_dac_edge.data(), 0.5);
        }
    }
}
