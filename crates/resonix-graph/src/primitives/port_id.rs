use core::ops::Deref;

use nohash_hasher::IsEnabled;

use crate::primitives::Id;

/// Strongly typed wrapper around the Id primitive for strong type-checking
#[derive(Copy, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PortId(Id);

impl IsEnabled for PortId {}

impl PortId {
    pub const fn new(id: usize) -> Self {
        Self(Id::new(id))
    }

    // TODO: should this number be larger? = 2^8
    // TODO: should this number be dynamic based on the actual
    // number of connections used in the graph at runtime?
    /// This represents the maximum number of connections that a node can have.
    /// Smaller = more memory efficient allocations for the Graph,
    /// since we allocate an array of this size to hold all the necessary connections.
    pub const MAX_PORT_ID: usize = 256;
}

// other convenience implementations possible here

impl Deref for PortId {
    type Target = Id;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I: Into<Id>> From<I> for PortId {
    fn from(value: I) -> Self {
        PortId(value.into())
    }
}

impl From<PortId> for usize {
    fn from(value: PortId) -> Self {
        **value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_value_accessible_via_double_deref() {
        let port_id = PortId::new(3);
        assert_eq!(**port_id, 3);
    }

    #[test]
    fn max_port_id_is_256() {
        assert_eq!(PortId::MAX_PORT_ID, 256);
    }

    #[test]
    fn from_usize_creates_port_id_with_that_value() {
        let port_id = PortId::from(6usize);
        assert_eq!(**port_id, 6);
    }

    #[test]
    fn port_ids_with_same_value_are_equal() {
        assert_eq!(PortId::new(2), PortId::new(2));
    }

    #[test]
    fn port_ids_with_different_values_are_not_equal() {
        assert_ne!(PortId::new(0), PortId::new(1));
    }

    #[test]
    fn lower_value_port_id_is_less_than_higher_value() {
        assert!(PortId::new(0) < PortId::new(255));
    }
}
