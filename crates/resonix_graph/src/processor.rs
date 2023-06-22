use std::{
    any::Any,
    collections::{HashSet, VecDeque},
    ptr::{addr_of, addr_of_mut},
};

use petgraph::{stable_graph::NodeIndex, visit::Dfs, Direction, Graph};

use uuid::Uuid;

use crate::{AddConnectionError, BoxedNode, Connection, DACNode, Node, NodeType};

#[derive(thiserror::Error, Debug)]
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
}

/// Cloning the audio context is an outward clone of the
/// audio context handle
#[derive(Debug, Default, Clone)]
pub struct Processor {
    graph: Graph<BoxedNode, Connection>,
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
    pub(crate) fn run(&mut self) {
        if self.visit_order.is_none() {
            self.initialize_visit_order();
        }

        for node_index in self.visit_order.as_ref().unwrap() {
            // some very unsafe shenanigans here, since Rust doesn't know that the immutable borrows
            // of Connections from the Graph below are not the same mutable borrow of the Node
            // this should be safe-ish since we are only borrowing connections above, and here we are only borrowing a node
            // let graph_ptr_mut = addr_of!(self.graph) as *mut Graph<BoxedNode, Connection>;
            // let graph_mut = unsafe { &mut *graph_ptr_mut as &mut Graph<BoxedNode, Connection> };
            // let node = &mut graph_mut[*node_index];
            let graph_ptr = addr_of!(self.graph);

            let node_mut: &mut Box<dyn Node> = &mut self.graph[*node_index];

            // it is safe to immutably borrow `node_mut` for its connections, AS LONG AS the
            // connections are not touched within the Node's `process` implementation
            // todo => refactor `Node::process` to make this impossible (and therefore actually safe)
            let node_ptr = addr_of!(*node_mut);
            let node = unsafe { &*node_ptr as &BoxedNode };
            let incoming_edge_indexes = node.incoming_connection_indexes();
            let outgoing_edge_indexes = node.outgoing_connection_indexes();

            let mut incoming_connections = {
                let graph_for_incoming_edges =
                    unsafe { &*(graph_ptr) as &Graph<BoxedNode, Connection> };
                incoming_edge_indexes.iter().map(|i| {
                    let connection = graph_for_incoming_edges.edge_weight(*i).unwrap();
                    let ptr = addr_of!(*connection);
                    unsafe { &*(ptr) as &Connection }
                })
            };

            let mut outgoing_connections = {
                let graph_mut_for_outgoing_edges = unsafe {
                    let ptr_mut = graph_ptr as *mut Graph<BoxedNode, Connection>;
                    &mut *(ptr_mut) as &mut Graph<BoxedNode, Connection>
                };
                outgoing_edge_indexes.iter().map(|i| {
                    let connection = graph_mut_for_outgoing_edges.edge_weight_mut(*i).unwrap();
                    let ptr = addr_of_mut!(*connection);
                    unsafe { &mut *(ptr) as &mut Connection }
                })
            };

            node_mut.process(&mut incoming_connections, &mut outgoing_connections)
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
            if incoming_connections.any(|incoming_connection| {
                !connection_visit_set.contains(incoming_connection.uuid())
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
                connection_visit_set.insert(*edge_reference.weight().uuid());
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
                self.graph[*i]
                    .as_any()
                    .downcast_ref::<DACNode>()
                    .unwrap()
                    .data()
            })
            .sum()
    }

    pub fn add_node<N: Node + 'static>(&mut self, node: N) -> Result<NodeIndex, N> {
        if self.node_uuids.contains(node.uuid()) {
            return Err(node);
        }

        let is_input = node.node_type() == NodeType::Input;

        let is_dac = {
            let node_as_any = &node as &dyn Any;
            node_as_any.downcast_ref::<DACNode>().is_some()
        };

        let node_index = self.graph.add_node(Box::new(node));

        if is_input {
            self.input_node_indexes.push(node_index);
        }

        if is_dac {
            self.dac_node_indexes.push(node_index);
        }

        Ok(node_index)
    }

    fn connect_with_indexes(
        &mut self,
        parent_node_index: NodeIndex,
        child_node_index: NodeIndex,
        from_index: usize,
        to_index: usize,
    ) -> Result<&mut Self, ConnectError> {
        // check if connection indexes are out of bounds
        {
            let parent_node =
                &self
                    .graph
                    .node_weight(parent_node_index)
                    .ok_or(ConnectError::NodeNotFound {
                        node_index: parent_node_index,
                    })?;
            let child_node =
                &self
                    .graph
                    .node_weight(child_node_index)
                    .ok_or(ConnectError::NodeNotFound {
                        node_index: child_node_index,
                    })?;

            Self::check_index_out_of_bounds(parent_node, child_node, from_index, to_index)?;
        }

        // add connection to graph
        let edge_index = self.graph.add_edge(
            parent_node_index,
            child_node_index,
            Connection::from_indexes(from_index, to_index),
        );

        // add connection indexes to nodes themselves for faster retrieval later
        self.graph
            .node_weight_mut(parent_node_index)
            .ok_or(ConnectError::NodeNotFound {
                node_index: parent_node_index,
            })?
            .add_outgoing_connection_index(edge_index)
            .unwrap();

        self.graph
            .node_weight_mut(child_node_index)
            .ok_or(ConnectError::NodeNotFound {
                node_index: child_node_index,
            })?
            .add_incoming_connection_index(edge_index)
            .unwrap();

        Ok(self)
    }

    pub fn connect(
        &mut self,
        node_1: NodeIndex,
        node_2: NodeIndex,
    ) -> Result<&mut Self, ConnectError> {
        self.connect_with_indexes(node_1, node_2, Default::default(), Default::default())
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
}

#[cfg(test)]
mod test_audio_context {}
