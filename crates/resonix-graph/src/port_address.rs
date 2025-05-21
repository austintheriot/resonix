use crate::{ResonixId, ResonixPortAddressDirection};

#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResonixPortAddress {
    node_id: ResonixId,
    port_id: ResonixId,
    direction: ResonixPortAddressDirection,
}

impl ResonixPortAddress {
    pub const fn new(
        node_id: ResonixId,
        port_id: ResonixId,
        direction: ResonixPortAddressDirection,
    ) -> Self {
        Self {
            node_id,
            port_id,
            direction,
        }
    }
}
