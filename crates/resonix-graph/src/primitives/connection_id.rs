use core::ops::Deref;

use nohash_hasher::IsEnabled;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::primitives::Id;

/// Strongly typed wrapper around the Id primitive for strong type-checking
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
#[derive(Copy, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConnectionId(Id);

impl IsEnabled for ConnectionId {}

impl ConnectionId {
    pub const fn new(id: usize) -> Self {
        Self(Id::new(id))
    }
}

// other convenience implementations possible here

impl Deref for ConnectionId {
    type Target = Id;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I: Into<Id>> From<I> for ConnectionId {
    fn from(value: I) -> Self {
        ConnectionId(value.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_value_accessible_via_double_deref() {
        let connection_id = ConnectionId::new(77);
        assert_eq!(**connection_id, 77);
    }

    #[test]
    fn from_usize_creates_connection_id_with_that_value() {
        let connection_id = ConnectionId::from(100usize);
        assert_eq!(**connection_id, 100);
    }

    #[test]
    fn connection_ids_with_same_value_are_equal() {
        assert_eq!(ConnectionId::new(5), ConnectionId::new(5));
    }

    #[test]
    fn connection_ids_with_different_values_are_not_equal() {
        assert_ne!(ConnectionId::new(3), ConnectionId::new(4));
    }

    #[test]
    fn lower_value_connection_id_is_less_than_higher_value() {
        assert!(ConnectionId::new(0) < ConnectionId::new(1));
    }
}
