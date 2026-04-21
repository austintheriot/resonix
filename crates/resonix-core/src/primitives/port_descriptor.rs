use crate::primitives::PortAddress;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PortDescriptor {
    pub address: PortAddress,
    pub channels: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::{NodeId, PortAddressDirection, PortId};

    fn make_port_descriptor(
        node_id_value: usize,
        port_id_value: usize,
        channels: usize,
    ) -> PortDescriptor {
        PortDescriptor {
            address: PortAddress::new(
                NodeId::new(node_id_value),
                PortId::new(port_id_value),
                PortAddressDirection::Input,
            ),
            channels,
        }
    }

    #[test]
    fn port_descriptors_with_same_address_and_channels_are_equal() {
        let descriptor_a = make_port_descriptor(1, 0, 2);
        let descriptor_b = make_port_descriptor(1, 0, 2);
        assert_eq!(descriptor_a, descriptor_b);
    }

    #[test]
    fn port_descriptors_with_different_channels_are_not_equal() {
        let mono_descriptor = make_port_descriptor(1, 0, 1);
        let stereo_descriptor = make_port_descriptor(1, 0, 2);
        assert_ne!(mono_descriptor, stereo_descriptor);
    }

    #[test]
    fn port_descriptors_with_different_addresses_are_not_equal() {
        let descriptor_for_node_1 = make_port_descriptor(1, 0, 1);
        let descriptor_for_node_2 = make_port_descriptor(2, 0, 1);
        assert_ne!(descriptor_for_node_1, descriptor_for_node_2);
    }

    #[test]
    fn port_descriptor_is_copy_and_clone() {
        let original = make_port_descriptor(0, 0, 1);
        let copied = original;
        let cloned = original;
        assert_eq!(copied, original);
        assert_eq!(cloned, original);
    }

    #[test]
    fn channels_field_is_accessible() {
        let descriptor = make_port_descriptor(0, 0, 4);
        assert_eq!(descriptor.channels, 4);
    }
}
