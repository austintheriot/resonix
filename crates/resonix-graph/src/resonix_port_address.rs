use crate::ResonixId;

#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResonixPortAddress {
    node_id: ResonixId,
    port_id: ResonixId,
}

impl ResonixPortAddress {
    pub const fn new(node_id: ResonixId, port_id: ResonixId) -> Self {
        Self { node_id, port_id }
    }
}
