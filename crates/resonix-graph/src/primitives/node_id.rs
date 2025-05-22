use core::ops::Deref;

use crate::ResonixId;

/// Strongly typed wrapper around the Id primitive for strong type-checking
#[derive(Copy, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId(ResonixId);

impl NodeId {
    pub const fn new(id: usize) -> Self {
        Self(ResonixId::new(id))
    }
}

// other convenience implementations possible here

impl Deref for NodeId {
    type Target = ResonixId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I: Into<ResonixId>> From<I> for NodeId {
    fn from(value: I) -> Self {
        NodeId(value.into())
    }
}
