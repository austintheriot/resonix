use core::ops::Deref;

use nohash_hasher::IsEnabled;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use wasm_bindgen::prelude::wasm_bindgen;

/// All Graph-internal audio is computed with `f32`s.
///
/// Conversion to other sample formats takes place on the
/// I/O boundaries.
#[derive(Copy, Default, Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
pub struct Sample(f32);

impl Sample {
    pub const fn new(id: f32) -> Self {
        Self(id)
    }
}

impl IsEnabled for Sample {}

// other convenience implementations possible here

impl Deref for Sample {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<i32> for Sample {
    fn from(value: i32) -> Self {
        Sample(value as f32)
    }
}

impl From<f32> for Sample {
    fn from(value: f32) -> Self {
        Sample(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_value_accessible_via_deref() {
        let sample = Sample::new(0.5f32);
        assert_eq!(*sample, 0.5f32);
    }

    #[test]
    fn default_sample_is_zero() {
        let sample = Sample::default();
        assert_eq!(*sample, 0.0f32);
    }

    #[test]
    fn from_f32_creates_sample_with_that_value() {
        let sample = Sample::from(1.0f32);
        assert_eq!(*sample, 1.0f32);
    }

    #[test]
    fn from_i32_casts_integer_to_float() {
        let sample = Sample::from(3i32);
        assert_eq!(*sample, 3.0f32);
    }

    #[test]
    fn samples_with_same_value_are_equal() {
        assert_eq!(Sample::new(2.0f32), Sample::new(2.0f32));
    }

    #[test]
    fn sample_with_lower_value_is_less_than_sample_with_higher_value() {
        assert!(Sample::new(-1.0f32) < Sample::new(1.0f32));
    }
}
