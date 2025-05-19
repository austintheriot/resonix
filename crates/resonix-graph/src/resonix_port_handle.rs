use crate::ResonixId;

pub struct ResonixPortHandle {
    node_id: ResonixId,
    port_id: ResonixId,
}

impl ResonixPortHandle {
    pub fn new(node_id: ResonixId, port_id: ResonixId) -> Self {
        Self { node_id, port_id }
    }
}
