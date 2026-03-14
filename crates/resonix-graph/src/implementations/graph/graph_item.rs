use crate::{implementations::graph::Node, primitives::Connection};

pub(in crate::implementations::graph) enum GraphItem {
    Node(Node),

    // TODO: add/remove this type once we know we need it
    #[allow(dead_code)]
    Connection(Connection),
}
