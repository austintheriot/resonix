use crate::{ResonixId, ResonixPortAddressDirection};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResonixPortAddress {
    node_id: ResonixId,
    port_id: ResonixId,
    port_address_direction: ResonixPortAddressDirection,
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
            port_address_direction: direction,
        }
    }

    pub const fn node_id(&self) -> ResonixId {
        self.node_id
    }

    pub const fn port_id(&self) -> ResonixId {
        self.port_id
    }

    pub const fn port_address_direction(&self) -> ResonixPortAddressDirection {
        self.port_address_direction
    }
}
