use crate::primitives::PortAddressDirection;

use super::{NodeId, PortId};

/// Indicates the exact connection address that a node is connected at
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PortAddress {
    node_id: NodeId,
    port_id: PortId,
    port_address_direction: PortAddressDirection,
}

impl PortAddress {
    pub const fn new(node_id: NodeId, port_id: PortId, direction: PortAddressDirection) -> Self {
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

    pub const fn port_address_direction(&self) -> PortAddressDirection {
        self.port_address_direction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_input_address(node_id_value: usize, port_id_value: usize) -> PortAddress {
        PortAddress::new(
            NodeId::new(node_id_value),
            PortId::new(port_id_value),
            PortAddressDirection::Input,
        )
    }

    fn make_output_address(node_id_value: usize, port_id_value: usize) -> PortAddress {
        PortAddress::new(
            NodeId::new(node_id_value),
            PortId::new(port_id_value),
            PortAddressDirection::Output,
        )
    }

    #[test]
    fn node_id_getter_returns_the_node_id_provided_at_construction() {
        let address = make_input_address(7, 0);
        assert_eq!(address.node_id(), NodeId::new(7));
    }

    #[test]
    fn port_id_getter_returns_the_port_id_provided_at_construction() {
        let address = make_input_address(0, 3);
        assert_eq!(address.port_id(), PortId::new(3));
    }

    #[test]
    fn direction_getter_returns_the_direction_provided_at_construction() {
        let input_address = make_input_address(0, 0);
        assert_eq!(
            input_address.port_address_direction(),
            PortAddressDirection::Input
        );

        let output_address = make_output_address(0, 0);
        assert_eq!(
            output_address.port_address_direction(),
            PortAddressDirection::Output
        );
    }

    #[test]
    fn addresses_with_same_components_are_equal() {
        let address_a = make_input_address(1, 0);
        let address_b = make_input_address(1, 0);
        assert_eq!(address_a, address_b);
    }

    #[test]
    fn addresses_with_different_node_ids_are_not_equal() {
        let address_a = make_input_address(1, 0);
        let address_b = make_input_address(2, 0);
        assert_ne!(address_a, address_b);
    }

    #[test]
    fn addresses_with_different_directions_are_not_equal() {
        let input_address = make_input_address(0, 0);
        let output_address = make_output_address(0, 0);
        assert_ne!(input_address, output_address);
    }

    #[test]
    fn port_address_is_copy_and_clone() {
        let original = make_input_address(5, 2);
        let copied = original;
        let cloned = original;
        assert_eq!(copied, original);
        assert_eq!(cloned, original);
    }
}
