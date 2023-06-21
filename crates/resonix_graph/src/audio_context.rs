use std::{
    any::Any,
    collections::{HashSet, VecDeque},
    hash::{Hash, Hasher},
    sync::{Arc, Mutex},
};

use petgraph::{
    stable_graph::NodeIndex,
    visit::{Dfs, IntoNodeIdentifiers},
    Direction, Graph,
};
#[cfg(feature = "cpal")]
use resonix_core::{DACBuildError, DACConfig, DAC};
use uuid::Uuid;

use crate::{BoxedNode, Connect, Connection, DACNode, Node, NodeType};

#[derive(Debug)]
struct AudioContextInner {
    graph: Graph<BoxedNode, Connection>,
    node_uuids: HashSet<Uuid>,
    visit_order: Option<Vec<NodeIndex>>,
    input_node_indexes: Vec<NodeIndex>,
    dac_node_indexes: Vec<NodeIndex>,
    uuid: Uuid,
}

impl AudioContextInner {
    fn run(&mut self) {
        if self.visit_order.is_none() {
            self.initialize_visit_order();
        }

        for node_index in self.visit_order.as_ref().unwrap() {
            let input_data: Vec<Connection> = self
                .graph
                .edges_directed(*node_index, Direction::Incoming)
                .map(|edge_reference| edge_reference.weight().clone())
                .collect();

            let mut outgoing_connections: Vec<Connection> = self
                .graph
                .edges_directed(*node_index, Direction::Outgoing)
                .map(|edge_reference| edge_reference.weight().clone())
                .filter(|outgoing_connection| !input_data.contains(outgoing_connection))
                .collect();

            let node = &mut self.graph[*node_index];

            node.process(input_data.as_slice(), outgoing_connections.as_mut_slice())
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

    fn input_nodes(&self) -> Vec<BoxedNode> {
        self.input_node_indexes
            .iter()
            .map(|i| dyn_clone::clone_box(&*self.graph[*i]))
            .collect()
    }

    #[cfg(feature = "cpal")]
    fn dac_nodes(&self) -> Vec<DACNode> {
        self.dac_node_indexes
            .iter()
            .filter_map(|i: &NodeIndex| {
                self.graph[*i]
                    .as_any()
                    .downcast_ref::<DACNode>()
                    .map(Clone::clone)
            })
            .collect()
    }

    fn add_node<N: Node + Clone + 'static>(&mut self, node: N) -> &mut Self {
        if self.node_uuids.contains(node.uuid()) {
            return self;
        }

        let is_input = node.node_type() == NodeType::Input;

        let node_index = self.graph.add_node(Box::new(node.clone()));

        if is_input {
            self.input_node_indexes.push(node_index);
        }

        {
            let node_as_any = &node as &dyn Any;
            if node_as_any.downcast_ref::<DACNode>().is_some() {
                self.dac_node_indexes.push(node_index)
            }
        }

        self
    }

    fn connect_nodes<N1: Node + Connect, N2: Node + Connect>(
        &mut self,
        node_1: N1,
        node_2: N2,
    ) -> &mut Self {
        self.connect_nodes_with_indexes(node_1, Default::default(), node_2, Default::default())
    }

    fn connect_nodes_with_indexes<N1: Node + Connect, N2: Node + Connect>(
        &mut self,
        node_1: N1,
        from_index: usize,
        node_2: N2,
        to_index: usize,
    ) -> &mut Self {
        let Some(node_index_1) = self.find_node_index(node_1.uuid()) else {
            // todo add error messaging here?
            return self;
        };
        let Some(node_index_2) = self.find_node_index(node_2.uuid()) else {
            // todo add error messaging here?
            return self;
        };

        self.graph.add_edge(
            node_index_1,
            node_index_2,
            Connection::from_indexes(from_index, to_index),
        );

        self
    }

    fn find_node_index(&self, uuid: &Uuid) -> Option<NodeIndex> {
        self.graph
            .node_identifiers()
            .map(|i| (i, &self.graph[i]))
            .find(|(_, weight)| weight.uuid() == uuid)
            .map(|(i, _)| i)
    }
}

impl Default for AudioContextInner {
    fn default() -> Self {
        AudioContextInner {
            graph: Graph::new(),
            uuid: Uuid::new_v4(),
            visit_order: None,
            input_node_indexes: Vec::new(),
            dac_node_indexes: Vec::new(),
            node_uuids: HashSet::new(),
        }
    }
}

#[cfg(test)]
mod test_audio_context_inner {
    use crate::{
        AudioContext, Connect, ConstantNode, MultiplyNode, Node, PassThroughNode, RecordNode,
        SineNode,
    };

    #[test]
    fn allows_running_audio_graph() {
        let mut audio_context = AudioContext::default();
        let constant_node_left = ConstantNode::new_with_signal_value(&mut audio_context, 4.0);
        let constant_node_right = ConstantNode::new_with_signal_value(&mut audio_context, 0.5);

        let pass_through_node_left = PassThroughNode::new(&mut audio_context);
        constant_node_left.connect(&pass_through_node_left).unwrap();
        let pass_through_node_right = PassThroughNode::new(&mut audio_context);
        constant_node_right
            .connect(&pass_through_node_right)
            .unwrap();

        let multiply_node = MultiplyNode::new(&mut audio_context);
        pass_through_node_left.connect(&multiply_node).unwrap();
        pass_through_node_right
            .connect_nodes_with_indexes(0, &multiply_node, 1)
            .unwrap();
        let record_node = RecordNode::new(&mut audio_context);
        multiply_node.connect(&record_node).unwrap();
        audio_context.run();

        // recording should now contain one sample
        {
            let record_data = record_node.data();
            assert_eq!(record_data.len(), 1);
            assert_eq!(*record_data.first().unwrap(), 2.0);
        }

        audio_context.run();

        // another sample should be recorded (with the same value)
        {
            let record_data = record_node.data();
            assert_eq!(record_data.len(), 2);
            assert_eq!(*record_data.get(1).unwrap(), 2.0);
        }
    }

    #[test]
    fn allows_getting_input_nodes() {
        let mut audio_context = AudioContext::new();
        let sine_node = SineNode::new(&mut audio_context);
        RecordNode::new(&mut audio_context);
        PassThroughNode::new(&mut audio_context);
        let constant_node = ConstantNode::new(&mut audio_context);
        MultiplyNode::new(&mut audio_context);

        let input_nodes = audio_context.input_nodes();

        assert_eq!(input_nodes.len(), 2);
        assert!(input_nodes
            .iter()
            .any(|node| node.uuid() == sine_node.uuid()));
        assert!(input_nodes
            .iter()
            .any(|node| node.uuid() == constant_node.uuid()));
    }

    #[cfg(feature = "cpal")]
    #[test]
    fn allows_getting_dac_nodes() {
        use crate::DACNode;

        let mut audio_context = AudioContext::new();
        SineNode::new(&mut audio_context);
        RecordNode::new(&mut audio_context);
        PassThroughNode::new(&mut audio_context);
        ConstantNode::new(&mut audio_context);
        MultiplyNode::new(&mut audio_context);
        let dac_node = DACNode::new(&mut audio_context);

        let dac_nodes = audio_context.dac_nodes();

        assert_eq!(dac_nodes.len(), 1);
        assert!(dac_nodes.iter().any(|node| node.uuid() == dac_node.uuid()));
    }
}

/// Cloning the audio context is an outward clone of the
/// audio context handle
#[derive(Debug, Clone)]
pub struct AudioContext {
    uuid: Uuid,
    audio_context_inner: Arc<Mutex<AudioContextInner>>,
    #[cfg(feature = "cpal")]
    dac: Arc<Mutex<Option<DAC>>>,
}

unsafe impl Send for AudioContext {}
unsafe impl Sync for AudioContext {}

impl AudioContext {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            audio_context_inner: Default::default(),
            dac: Default::default(),
        }
    }

    pub fn input_nodes(&self) -> Vec<BoxedNode> {
        self.audio_context_inner.lock().unwrap().input_nodes()
    }

    #[cfg(feature = "cpal")]
    pub fn dac_nodes(&self) -> Vec<DACNode> {
        self.audio_context_inner.lock().unwrap().dac_nodes()
    }

    #[cfg(feature = "cpal")]
    pub fn num_channels(&self) -> Option<u16> {
        self.dac
            .lock()
            .unwrap()
            .as_ref()
            .map(|dac| dac.config.stream_config.channels)
    }

    #[cfg(feature = "cpal")]
    pub fn sample_rate(&self) -> Option<u32> {
        self.dac
            .lock()
            .unwrap()
            .as_ref()
            .map(|dac| dac.config.stream_config.sample_rate.0)
    }

    pub fn run(&mut self) {
        self.audio_context_inner.lock().unwrap().run();
    }

    #[cfg(feature = "cpal")]
    pub async fn initialize_dac_from_defaults(&mut self) -> Result<&mut Self, DACBuildError> {
        let audio_context_inner = Arc::clone(&self.audio_context_inner);
        let dac = DAC::from_dac_defaults(move |buffer: &mut [f32], config: Arc<DACConfig>| {
            let mut audio_context_inner = audio_context_inner.lock().unwrap();
            let num_channels = config.stream_config.channels as usize;
            let dac_nodes = audio_context_inner.dac_nodes();

            for frame in buffer.chunks_mut(num_channels) {
                audio_context_inner.run();
                let sample_sim = dac_nodes.iter().map(|node| node.data()).sum();
                for channel in frame.iter_mut() {
                    *channel = cpal::Sample::from::<f32>(&sample_sim);
                }
            }
        })
        .await?;
        self.dac.lock().unwrap().replace(dac);

        Ok(self)
    }

    #[cfg(feature = "cpal")]
    pub async fn initialize_dac_from_config(
        &mut self,
        _dac_config: DACConfig,
    ) -> Result<&mut Self, DACBuildError> {
        todo!()
    }

    pub fn uninitialize_dac(&mut self) -> Option<DAC> {
        self.dac.lock().unwrap().take()
    }

    pub(crate) fn add_node<N: Node + Clone + 'static>(&mut self, node: N) -> &mut Self {
        self.audio_context_inner.lock().unwrap().add_node(node);
        self
    }

    pub(crate) fn connect_nodes<N1: Node + Connect, N2: Node + Connect>(
        &self,
        node_1: N1,
        node_2: N2,
    ) -> &Self {
        self.audio_context_inner
            .lock()
            .unwrap()
            .connect_nodes(node_1, node_2);
        self
    }

    pub(crate) fn connect_nodes_with_indexes<N1: Node + Connect, N2: Node + Connect>(
        &self,
        node_1: N1,
        from_index: usize,
        node_2: N2,
        to_index: usize,
    ) -> &Self {
        self.audio_context_inner
            .lock()
            .unwrap()
            .connect_nodes_with_indexes(node_1, from_index, node_2, to_index);
        self
    }
}

impl PartialEq for AudioContext {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for AudioContext {}

impl PartialOrd for AudioContext {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.uuid.partial_cmp(&other.uuid)
    }
}

impl Ord for AudioContext {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uuid.cmp(&other.uuid)
    }
}

impl Hash for AudioContext {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}

impl Default for AudioContext {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            audio_context_inner: Arc::new(Mutex::new(AudioContextInner::default())),
            dac: Arc::new(Mutex::new(None)),
        }
    }
}

#[cfg(test)]
mod test_audio_context {}
