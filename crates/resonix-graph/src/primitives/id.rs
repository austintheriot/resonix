use core::ops::Deref;

use nohash_hasher::IsEnabled;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use wasm_bindgen::prelude::wasm_bindgen;

/// Basic id for data structures around the Graph
#[derive(Copy, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
pub struct Id(usize);

impl Id {
    pub const fn new(id: usize) -> Self {
        Self(id)
    }
}

impl IsEnabled for Id {}

// other convenience implementations possible here

impl Deref for Id {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<i32> for Id {
    fn from(value: i32) -> Self {
        Id(value as usize)
    }
}

impl From<usize> for Id {
    fn from(value: usize) -> Self {
        Id(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_value_accessible_via_deref() {
        let id = Id::new(42);
        assert_eq!(*id, 42);
    }

    #[test]
    fn from_usize_creates_id_with_that_value() {
        let id = Id::from(7usize);
        assert_eq!(*id, 7);
    }

    #[test]
    fn from_i32_casts_to_usize() {
        let id = Id::from(10i32);
        assert_eq!(*id, 10);
    }

    #[test]
    fn ids_with_same_value_are_equal() {
        assert_eq!(Id::new(5), Id::new(5));
    }

    #[test]
    fn ids_with_different_values_are_not_equal() {
        assert_ne!(Id::new(1), Id::new(2));
    }

    #[test]
    fn lower_value_id_is_less_than_higher_value_id() {
        assert!(Id::new(1) < Id::new(2));
    }

    #[test]
    fn id_is_copy_and_clone() {
        let original = Id::new(3);
        let copied = original;
        let cloned = original;
        assert_eq!(copied, original);
        assert_eq!(cloned, original);
    }
}
