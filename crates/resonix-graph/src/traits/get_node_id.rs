use crate::primitives::ResonixId;

pub trait GetNodeId {
    fn node_id(&self) -> ResonixId;
}
