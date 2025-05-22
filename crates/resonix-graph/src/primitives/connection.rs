use crate::{GenerateId, ResonixPortAddress};

use super::ConnectionId;

#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResonixConnection {
    pub connection_id: ConnectionId,
    pub start_port_address: ResonixPortAddress,
    pub end_port_address: ResonixPortAddress,
}

/// Provides details about where specifically data should be routed between two nodes
impl ResonixConnection {
    pub fn new<G: GenerateId>(
        id_generator: &mut G,
        start_port_address: ResonixPortAddress,
        end_port_address: ResonixPortAddress,
    ) -> Self {
        let connection_id = ConnectionId::from(id_generator.generate_id());
        Self {
            connection_id,
            start_port_address,
            end_port_address,
        }
    }
}
