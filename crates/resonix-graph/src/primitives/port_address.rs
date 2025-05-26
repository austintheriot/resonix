use crate::primitives::ResonixPortAddressDirection;

use super::{NodeId, PortId};

/// Indicates the exact connection address that a node is connected at
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PortAddress {
    node_id: NodeId,
    port_id: PortId,
    port_address_direction: ResonixPortAddressDirection,
}

impl PortAddress {
    pub const fn new(
        node_id: NodeId,
        port_id: PortId,
        direction: ResonixPortAddressDirection,
    ) -> Self {
        Self {
            node_id,
            port_id,
            port_address_direction: direction,
        }
    }

    pub const fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub const fn port_id(&self) -> PortId {
        self.port_id
    }

    pub const fn port_address_direction(&self) -> ResonixPortAddressDirection {
        self.port_address_direction
    }
}
