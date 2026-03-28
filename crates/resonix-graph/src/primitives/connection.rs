#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::primitives::PortAddress;
use crate::traits::GenerateId;

use super::ConnectionId;

#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::{NodeId, PortAddressDirection, PortId};
    use crate::test_utils::TestIdGenerator;

    fn make_output_address(node_id_value: usize) -> PortAddress {
        PortAddress::new(
            NodeId::new(node_id_value),
            PortId::new(0),
            PortAddressDirection::Output,
        )
    }

    fn make_input_address(node_id_value: usize) -> PortAddress {
        PortAddress::new(
            NodeId::new(node_id_value),
            PortId::new(0),
            PortAddressDirection::Input,
        )
    }

    #[test]
    fn new_stores_start_and_end_port_addresses() {
        let mut id_generator = TestIdGenerator(0);
        let start_address = make_output_address(1);
        let end_address = make_input_address(2);

        let connection = Connection::new(&mut id_generator, start_address, end_address);

        assert_eq!(connection.start_port_address, start_address);
        assert_eq!(connection.end_port_address, end_address);
    }

    #[test]
    fn new_generates_connection_id_from_id_generator() {
        let mut id_generator = TestIdGenerator(0);
        let connection = Connection::new(
            &mut id_generator,
            make_output_address(0),
            make_input_address(1),
        );

        // TestIdGenerator(0) yields id=0 on first call
        assert_eq!(connection.connection_id, ConnectionId::new(0));
    }

    #[test]
    fn successive_connections_get_distinct_ids() {
        let mut id_generator = TestIdGenerator(0);
        let connection_a = Connection::new(
            &mut id_generator,
            make_output_address(0),
            make_input_address(1),
        );
        let connection_b = Connection::new(
            &mut id_generator,
            make_output_address(2),
            make_input_address(3),
        );

        assert_ne!(connection_a.connection_id, connection_b.connection_id);
    }
}
