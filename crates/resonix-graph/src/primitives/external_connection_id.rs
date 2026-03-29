use core::ops::Deref;

use nohash_hasher::IsEnabled;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::primitives::Id;

/// Strongly typed wrapper around the Id primitive
/// Indicates that a Node's connection for a given port is an
/// external one (that is, it connects to I/O for data)
///
/// Maintaining separate internal/external connection_ids also
/// enables passing in I/O connection data densely (in slices)
#[derive(Copy, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct ExternalConnectionId(Id);

impl IsEnabled for ExternalConnectionId {}

impl ExternalConnectionId {
    pub const fn new(id: usize) -> Self {
        Self(Id::new(id))
    }
}

// other convenience implementations possible here

impl Deref for ExternalConnectionId {
    type Target = Id;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I: Into<Id>> From<I> for ExternalConnectionId {
    fn from(value: I) -> Self {
        ExternalConnectionId(value.into())
    }
}

impl From<ExternalConnectionId> for usize {
    fn from(value: ExternalConnectionId) -> Self {
        **value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_value_accessible_via_double_deref() {
        let external_connection_id = ExternalConnectionId::new(77);
        assert_eq!(**external_connection_id, 77);
    }

    #[test]
    fn from_usize_creates_external_connection_id_with_that_value() {
        let external_connection_id = ExternalConnectionId::from(100usize);
        assert_eq!(**external_connection_id, 100);
    }

    #[test]
    fn external_connection_ids_with_same_value_are_equal() {
        assert_eq!(ExternalConnectionId::new(5), ExternalConnectionId::new(5));
    }

    #[test]
    fn external_connection_ids_with_different_values_are_not_equal() {
        assert_ne!(ExternalConnectionId::new(3), ExternalConnectionId::new(4));
    }

    #[test]
    fn lower_value_external_connection_id_is_less_than_higher_value() {
        assert!(ExternalConnectionId::new(0) < ExternalConnectionId::new(1));
    }
}
