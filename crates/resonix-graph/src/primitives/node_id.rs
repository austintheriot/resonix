use core::ops::Deref;

use nohash_hasher::IsEnabled;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::primitives::Id;

/// Strongly typed wrapper around the Id primitive for strong type-checking
#[derive(Copy, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct NodeId(Id);

impl NodeId {
    pub const fn new(id: usize) -> Self {
        Self(Id::new(id))
    }
}

// other convenience implementations possible here

impl IsEnabled for NodeId {}

impl Deref for NodeId {
    type Target = Id;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I: Into<Id>> From<I> for NodeId {
    fn from(value: I) -> Self {
        NodeId(value.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_value_accessible_via_double_deref() {
        let node_id = NodeId::new(99);
        assert_eq!(**node_id, 99);
    }

    #[test]
    fn from_usize_creates_node_id_with_that_value() {
        let node_id = NodeId::from(15usize);
        assert_eq!(**node_id, 15);
    }

    #[test]
    fn from_i32_casts_through_id_to_usize() {
        let node_id = NodeId::from(20i32);
        assert_eq!(**node_id, 20);
    }

    #[test]
    fn node_ids_with_same_value_are_equal() {
        assert_eq!(NodeId::new(4), NodeId::new(4));
    }

    #[test]
    fn node_ids_with_different_values_are_not_equal() {
        assert_ne!(NodeId::new(1), NodeId::new(2));
    }

    #[test]
    fn lower_value_node_id_is_less_than_higher_value() {
        assert!(NodeId::new(0) < NodeId::new(1));
    }

    #[test]
    fn node_id_is_copy_and_clone() {
        let original = NodeId::new(8);
        let copied = original;
        let cloned = original;
        assert_eq!(copied, original);
        assert_eq!(cloned, original);
    }
}
