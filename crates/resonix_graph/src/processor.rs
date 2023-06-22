use std::{
    any::Any,
    collections::{HashSet, VecDeque},
    ptr::addr_of,
};

use petgraph::{
    graph::EdgeReference,
    stable_graph::NodeIndex,
    visit::{Dfs, IntoNodeIdentifiers},
    Direction, Graph,
};

use uuid::Uuid;

use crate::{BoxedNode, ConnectError, Connection, DACNode, Node, NodeType};

#[cfg(test)]
mod test_audio_context_inner {

    #[test]
    fn allows_running_audio_graph() {
        todo!()
        // let mut audio_context = Processor::default();
        // let constant_node_left = ConstantNode::new_with_signal_value(&mut audio_context, 4.0);
        // let constant_node_right = ConstantNode::new_with_signal_value(&mut audio_context, 0.5);

        // let pass_through_node_left = PassThroughNode::new(&mut audio_context);
        // constant_node_left.connect(&pass_through_node_left).unwrap();
        // let pass_through_node_right = PassThroughNode::new(&mut audio_context);
        // constant_node_right
        //     .connect(&pass_through_node_right)
        //     .unwrap();

        // let multiply_node = MultiplyNode::new(&mut audio_context);
        // pass_through_node_left.connect(&multiply_node).unwrap();
        // pass_through_node_right
        //     .connect_nodes_with_indexes(0, &multiply_node, 1)
        //     .unwrap();
        // let record_node = RecordNode::new(&mut audio_context);
        // multiply_node.connect(&record_node).unwrap();
        // audio_context.run();

        // // recording should now contain one sample
        // {
        //     let record_data = record_node.data();
        //     assert_eq!(record_data.len(), 1);
        //     assert_eq!(*record_data.first().unwrap(), 2.0);
        // }

        // audio_context.run();

        // // another sample should be recorded (with the same value)
        // {
        //     let record_data = record_node.data();
        //     assert_eq!(record_data.len(), 2);
        //     assert_eq!(*record_data.get(1).unwrap(), 2.0);
        // }
    }

    #[test]
    fn allows_getting_input_nodes() {
        todo!()
        // let mut audio_context = Processor::new();
        // let sine_node = SineNode::new(&mut audio_context);
        // RecordNode::new(&mut audio_context);
        // PassThroughNode::new(&mut audio_context);
        // let constant_node = ConstantNode::new(&mut audio_context);
        // MultiplyNode::new(&mut audio_context);

        // let input_nodes = audio_context.input_nodes();

        // assert_eq!(input_nodes.len(), 2);
        // assert!(input_nodes
        //     .iter()
        //     .any(|node| node.uuid() == sine_node.uuid()));
        // assert!(input_nodes
        //     .iter()
        //     .any(|node| node.uuid() == constant_node.uuid()));
    }

    #[cfg(feature = "dac")]
    #[test]
    fn allows_getting_dac_nodes() {
        todo!()
        // use crate::DACNode;

        // let mut audio_context = Processor::new();
        // SineNode::new(&mut audio_context);
        // RecordNode::new(&mut audio_context);
        // PassThroughNode::new(&mut audio_context);
        // ConstantNode::new(&mut audio_context);
        // MultiplyNode::new(&mut audio_context);
        // let dac_node = DACNode::new(&mut audio_context);

        // let dac_nodes = audio_context.dac_nodes();

        // assert_eq!(dac_nodes.len(), 1);
        // assert!(dac_nodes.iter().any(|node| node.uuid() == dac_node.uuid()));
    }
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

    pub(crate) fn run(&mut self) {
        if self.visit_order.is_none() {
            self.initialize_visit_order();
        }

        for node_index in self.visit_order.as_ref().unwrap() {
            let mut incoming_connections = self
                .graph
                .edges_directed(*node_index, Direction::Incoming)
                .map(|edge_reference| edge_reference.weight());

            let mut outgoing_connections = self
                .graph
                .edges_directed(*node_index, Direction::Outgoing)
                // NODE: if `resonix` should ever support cyclic graphs, it will be imperative
                // to filter outgoing_connections so that we are not creating
                // multiple mutable references to the same Connection
                .map(|edge_reference: EdgeReference<'_, Connection>| edge_reference.weight())
                .map(|connection| unsafe {
                    // `petgraph` provides no way of obtaining a mutable reference to the edges
                    // from a node. todo - move this step into a pre-compute function to keep
                    // this hot path clear
                    let ptr_mut = addr_of!(*connection) as *mut Connection;
                    &mut (*ptr_mut) as &mut Connection
                });

            // some very unsafe shenanigans here, since Rust doesn't know that the immutable borrows values from the Graph (above)
            // are not the same parts of the graph that are borrowed mutably here
            // this should be safe-ish since we are only borrowing connections above, and here we are only borrowing a node
            let graph_ptr_mut = addr_of!(self.graph) as *mut Graph<BoxedNode, Connection>;
            let graph_mut = unsafe { &mut *graph_ptr_mut as &mut Graph<BoxedNode, Connection> };
            let node = &mut graph_mut[*node_index];

            node.process(&mut incoming_connections, &mut outgoing_connections)
        }
    }

    fn initialize_visit_order(&mut self) {
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
                !connection_visit_set.contains(&incoming_connection.uuid)
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
                connection_visit_set.insert(edge_reference.weight().uuid);
            });
        }

        self.visit_order = Some(final_visit_order);
    }

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

        self.graph.add_edge(
            parent_node_index,
            child_node_index,
            Connection::from_indexes(from_index, to_index),
        );

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

    fn find_node_index(&self, uuid: &Uuid) -> Option<NodeIndex> {
        self.graph
            .node_identifiers()
            .map(|i| (i, &self.graph[i]))
            .find(|(_, weight)| weight.uuid() == uuid)
            .map(|(i, _)| i)
    }
}

#[cfg(test)]
mod test_audio_context {}
