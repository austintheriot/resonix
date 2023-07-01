use std::{ops::{Deref, DerefMut}, rc::Rc, cell::RefCell};

use petgraph::{Graph, data::{FromElements, Element}, visit::GraphBase};

use crate::{BoxedNode, Connection};

type NonSharedGraphInner = Graph<BoxedNode, Connection>;

#[derive(Debug, Default, Clone)]
pub struct NonSharedGraph(NonSharedGraphInner);

impl Deref for NonSharedGraph {
    type Target = NonSharedGraphInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NonSharedGraph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

type SharedGraphInner = Graph<Rc<RefCell<BoxedNode>>, Rc<RefCell<Connection>>>;

#[derive(Debug, Default, Clone)]
pub struct SharedGraph(SharedGraphInner);

impl Deref for SharedGraph {
    type Target = SharedGraphInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SharedGraph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<NonSharedGraph> for SharedGraph {
    fn from(value: NonSharedGraph) -> Self {
        let (nodes, edges) = value.into_nodes_edges();
        let nodes = nodes.into_iter().map(|node| {
            let new_weight = Rc::new(RefCell::new(node.weight));
            Element::Node { weight: new_weight }
        });
        let edges = edges.into_iter().map(|edge| {
            let new_edge_weight = Rc::new(RefCell::new(edge.weight));
            Element::Edge { source: edge.source().index(), target: edge.target().index(), weight: new_edge_weight }
        });
        let elements = nodes.chain(edges);
        SharedGraph(Graph::from_elements(elements))
    }
}
