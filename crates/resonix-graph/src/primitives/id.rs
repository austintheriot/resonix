use core::ops::Deref;

use nohash_hasher::IsEnabled;

/// Basic id for data structures around the Graph
#[derive(Copy, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
