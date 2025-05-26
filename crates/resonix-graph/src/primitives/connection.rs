use crate::primitives::PortAddress;
use crate::traits::GenerateId;

use super::ConnectionId;

#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Connection {
    pub connection_id: ConnectionId,
    pub start_port_address: PortAddress,
    pub end_port_address: PortAddress,
}

/// Provides details about where specifically data should be routed between two nodes
impl Connection {
    pub fn new<G: GenerateId>(
        id_generator: &mut G,
        start_port_address: PortAddress,
        end_port_address: PortAddress,
    ) -> Self {
        let connection_id = ConnectionId::from(id_generator.generate_id());
        Self {
            connection_id,
            start_port_address,
            end_port_address,
        }
    }
}
