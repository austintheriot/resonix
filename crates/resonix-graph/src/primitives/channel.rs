use core::ops::Deref;

use nohash_hasher::IsEnabled;

/// Strongly-typed wrapper
/// Functions as an indexer for the channels of an audio buffer
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
