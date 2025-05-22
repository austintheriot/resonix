use crate::ResonixPortAddress;

#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResonixConnection {
    pub start_port_address: ResonixPortAddress,
    pub end_port_address: ResonixPortAddress,
}

/// Provides details about where specifically data should be routed between two nodes
impl ResonixConnection {
    pub fn new(
        start_port_address: ResonixPortAddress,
        end_port_address: ResonixPortAddress,
    ) -> Self {
        Self {
            start_port_address,
            end_port_address,
        }
    }
}
