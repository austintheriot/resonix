use core::ops::Deref;

use nohash_hasher::IsEnabled;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use wasm_bindgen::prelude::wasm_bindgen;

/// Strongly-typed wrapper
/// Functions as an indexer for the channels of an audio buffer
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
#[derive(Copy, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Channel(usize);

impl Channel {
    pub const fn new(id: usize) -> Self {
        Self(id)
    }
}

impl IsEnabled for Channel {}

// other convenience implementations possible here

impl Deref for Channel {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<i32> for Channel {
    fn from(value: i32) -> Self {
        Channel(value as usize)
    }
}

impl From<usize> for Channel {
    fn from(value: usize) -> Self {
        Channel(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_value_accessible_via_deref() {
        let channel = Channel::new(2);
        assert_eq!(*channel, 2);
    }

    #[test]
    fn from_usize_creates_channel_with_that_value() {
        let channel = Channel::from(3usize);
        assert_eq!(*channel, 3);
    }

    #[test]
    fn from_i32_casts_to_usize() {
        let channel = Channel::from(0i32);
        assert_eq!(*channel, 0);
    }

    #[test]
    fn channels_with_same_value_are_equal() {
        assert_eq!(Channel::new(1), Channel::new(1));
    }

    #[test]
    fn channels_with_different_values_are_not_equal() {
        assert_ne!(Channel::new(0), Channel::new(1));
    }

    #[test]
    fn lower_channel_is_less_than_higher_channel() {
        assert!(Channel::new(0) < Channel::new(1));
    }
}
