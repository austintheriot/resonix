use core::ops::Deref;

use crate::primitives::Id;

/// Strongly typed wrapper around the Id primitive for strong type-checking
#[derive(Copy, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BufferId(Id);

impl BufferId {
    pub const fn new(id: usize) -> Self {
        Self(Id::new(id))
    }
}

// other convenience implementations possible here

impl Deref for BufferId {
    type Target = Id;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I: Into<Id>> From<I> for BufferId {
    fn from(value: I) -> Self {
        BufferId(value.into())
    }
}
