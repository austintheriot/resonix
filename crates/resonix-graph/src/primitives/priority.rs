use core::ops::Deref;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use wasm_bindgen::prelude::wasm_bindgen;

/// Strongly typed wrapper around a usize for better type checking
#[derive(Copy, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
pub struct Priority(usize);

impl Priority {
    pub const fn new(priority: usize) -> Self {
        Self(priority)
    }
}

// other convenience implementations possible here

impl Deref for Priority {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I: Into<usize>> From<I> for Priority {
    fn from(value: I) -> Self {
        Priority(value.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_value_accessible_via_deref() {
        let priority = Priority::new(10);
        assert_eq!(*priority, 10);
    }

    #[test]
    fn from_usize_creates_priority_with_that_value() {
        let priority = Priority::from(5usize);
        assert_eq!(*priority, 5);
    }

    #[test]
    fn priorities_with_same_value_are_equal() {
        assert_eq!(Priority::new(3), Priority::new(3));
    }

    #[test]
    fn lower_priority_value_is_less_than_higher_priority_value() {
        assert!(Priority::new(0) < Priority::new(100));
    }
}
