use crate::primitives::Id;

pub trait GetNodeId {
    fn node_id(&self) -> Id;
}
