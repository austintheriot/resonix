use core::ops::Deref;

use nohash_hasher::IsEnabled;

/// All Graph-internal audio is computed with `f32`s.
///
/// Conversion to other sample formats takes place on the
/// I/O boundaries.
#[derive(Copy, Default, Debug, Clone, PartialEq, PartialOrd)]
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
