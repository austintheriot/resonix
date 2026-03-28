#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

/// Indicates which direction a port can give/receive information
///
/// If Input, the port accepts data at that location, if Output,
/// the Node sends out information at that location.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub enum PortAddressDirection {
    /// Node accepts data from other Nodes at this port location
    Input,
    /// Node emits data to other Nodes at this port loation
    Output,
    /// Node emits data to the external system at this port location
    ExternalOutput,
    /// Node accepts data from the external system at this port location
    ExternalInput,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_four_variants_exist_and_are_distinct() {
        let directions = [
            PortAddressDirection::Input,
            PortAddressDirection::Output,
            PortAddressDirection::ExternalOutput,
            PortAddressDirection::ExternalInput,
        ];
        // Every pair should be distinct
        for i in 0..directions.len() {
            for j in 0..directions.len() {
                if i == j {
                    assert_eq!(directions[i], directions[j]);
                } else {
                    assert_ne!(directions[i], directions[j]);
                }
            }
        }
    }

    #[test]
    fn port_address_direction_is_copy_and_clone() {
        let original = PortAddressDirection::Input;
        let copied = original;
        let cloned = original;
        assert_eq!(copied, original);
        assert_eq!(cloned, original);
    }
}
