use core::ops::Deref;

use nohash_hasher::IsEnabled;

use crate::primitives::Id;

/// Strongly typed wrapper around the Id primitive for strong type-checking
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
