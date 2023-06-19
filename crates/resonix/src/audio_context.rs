use std::{
    cell::{Ref, RefCell, RefMut},
    collections::{HashSet, VecDeque},
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    rc::Rc,
};

use petgraph::{
    data::Build,
    stable_graph::NodeIndex,
    visit::{Bfs, IntoNodeIdentifiers},
    Direction, Graph,
};
use uuid::Uuid;

use crate::{BoxedNode, Connect, Connection, Node, NodeType};

#[derive(Debug)]
struct AudioContextInner {
    graph: Graph<RefCell<BoxedNode>, RefCell<Connection>>,
    node_uuids: HashSet<Uuid>,
    visit_order: Option<Vec<NodeIndex>>,
    inputs: Vec<NodeIndex>,
    uuid: Uuid,
}

impl AudioContextInner {
    fn run(&mut self) {
        if self.visit_order.is_none() {
            self.initialize_visit_order();
        }

        for node_index in self.visit_order.as_ref().unwrap() {
            let input_data: Vec<Ref<'_, Connection>> = self
                .graph
                .edges_directed(*node_index, Direction::Incoming)
                .map(|edge_reference| edge_reference.weight().borrow())
                .collect();

            let mut outgoing_connections: Vec<RefMut<'_, Connection>> = self
                .graph
                .edges_directed(*node_index, Direction::Outgoing)
                .map(|edge_reference| edge_reference.weight().borrow_mut())
                .collect();

            let mut node = (&self.graph[*node_index]).borrow_mut();

            node.process(&input_data, &mut outgoing_connections)
        }
    }

    fn initialize_visit_order(&mut self) {
        // if there are no inputs to the graph, then there is nothing to traverse
        if self.inputs.is_empty() {
            self.visit_order = Some(Vec::new());
        }

        let mut in_progress_visit_order = VecDeque::with_capacity(self.graph.node_count());
        let mut final_visit_order = Vec::with_capacity(self.graph.node_count());
        let mut bfs_search = Bfs::new(&self.graph, self.inputs[0]);

        // initialize bfs
        while let Some(node_index) = bfs_search.next(&self.graph) {
            in_progress_visit_order.push_back(node_index);
        }

        // find a valid path through graph, such that all inputs
        // are initialized for each node before that node's `process` function is run
        while let Some(node_index) = in_progress_visit_order.pop_front() {
            let mut incoming_connections = self
                .graph
                .edges_directed(node_index, Direction::Incoming)
                .map(|edge_reference| edge_reference.weight().borrow());

            // skip for now if inputs have not been initialized
            if incoming_connections.any(|incoming_connection| !incoming_connection.init) {
                in_progress_visit_order.push_back(node_index);
                continue;
            }

            final_visit_order.push(node_index);

            // if here, that means all inputs have been
            // initialized (or Node requires no inputs),
            // so we can initialize all outgoing connections
            self.graph
                .edges_directed(node_index, Direction::Outgoing)
                .for_each(|edge_reference| edge_reference.weight().borrow_mut().init = true);
        }

        self.visit_order = Some(final_visit_order);
    }

    pub(crate) fn add_node(&mut self, node: RefCell<BoxedNode>) -> &mut Self {
        if self.node_uuids.contains(&node.borrow().uuid()) {
            return self;
        }

        let is_input = node.borrow().node_type() == NodeType::Input;

        let node_index = self.graph.add_node(node);

        if is_input {
            self.inputs.push(node_index);
        }

        self
    }

    pub(crate) fn connect_nodes<N1: Node + Connect, N2: Node + Connect>(
        &mut self,
        node_1: N1,
        node_2: N2,
    ) -> &mut Self {
        self.connect_nodes_with_indexes(node_1, Default::default(), node_2, Default::default())
    }

    pub(crate) fn connect_nodes_with_indexes<N1: Node + Connect, N2: Node + Connect>(
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

        self.graph.update_edge(
            node_index_1,
            node_index_2,
            RefCell::new(Connection::from_indexes(from_index, to_index)),
        );

        self
    }

    fn find_node_index(&self, uuid: &Uuid) -> Option<NodeIndex> {
        self.graph
            .node_identifiers()
            .map(|i| (i, &self.graph[i]))
            .find(|(i, weight)| weight.borrow().uuid() == uuid)
            .map(|(i, _)| i)
    }
}

impl Default for AudioContextInner {
    fn default() -> Self {
        AudioContextInner {
            graph: Graph::new(),
            uuid: Uuid::new_v4(),
            visit_order: None,
            inputs: Vec::new(),
            node_uuids: HashSet::new(),
        }
    }
}

#[cfg(test)]
mod test_audio_context_inner {
    use crate::{AudioContext, Connect, MultiplyNode, PassThroughNode, RecordNode, SineNode};

    #[test]
    fn allows_creating_audio_graph() {
        let mut audio_context = AudioContext::default();
        let sine_node = SineNode::new_with_config(&mut audio_context, 44100, 440.0);
        let pass_through_node_left = PassThroughNode::new(&mut audio_context);
        let pass_through_node_right = PassThroughNode::new(&mut audio_context);
        let multiply_node = MultiplyNode::new(&mut audio_context);
        let record_node = RecordNode::new(&mut audio_context);
        sine_node.connect(&pass_through_node_left);
        sine_node.connect(&pass_through_node_right);
        pass_through_node_left.connect(&multiply_node);
        pass_through_node_right.connect_nodes_with_indexes(0, &multiply_node, 1);
        multiply_node.connect(&record_node);
        audio_context.run();

        // recording should now contain one sample
        {
            let record_data = record_node.data();
            assert_eq!(record_data.len(), 1);
            assert_eq!(*record_data.first().unwrap(), 0.0);
        }

        audio_context.run();

        {
            let record_data = record_node.data();
            assert_eq!(record_data.len(), 2);
            assert_eq!(*record_data.get(1).unwrap(), 0.0);
        }
    }
}

/// Cloning the audio context is an outward clone of the
/// audio context handle
#[derive(Debug, Clone)]
pub struct AudioContext {
    uuid: Uuid,
    audio_context_inner: Rc<RefCell<AudioContextInner>>,
}

impl AudioContext {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            audio_context_inner: Default::default(),
        }
    }

    pub fn run(&mut self) {
        self.inner_mut().run();
    }

    pub(crate) fn add_node(&mut self, node: RefCell<BoxedNode>) -> &mut Self {
        self.inner_mut().add_node(node);
        self
    }

    pub(crate) fn connect_nodes<N1: Node + Connect, N2: Node + Connect>(
        &self,
        node_1: N1,
        node_2: N2,
    ) -> &Self {
        self.inner_mut().connect_nodes(node_1, node_2);
        self
    }

    pub(crate) fn connect_nodes_with_indexes<N1: Node + Connect, N2: Node + Connect>(
        &self,
        node_1: N1,
        from_index: usize,
        node_2: N2,
        to_index: usize,
    ) -> &Self {
        self.inner_mut()
            .connect_nodes_with_indexes(node_1, from_index, node_2, to_index);
        self
    }

    fn inner(&self) -> Ref<AudioContextInner> {
        self.audio_context_inner.borrow()
    }

    fn inner_mut(&self) -> RefMut<AudioContextInner> {
        self.audio_context_inner.borrow_mut()
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
            audio_context_inner: Rc::new(RefCell::new(AudioContextInner::default())),
        }
    }
}
