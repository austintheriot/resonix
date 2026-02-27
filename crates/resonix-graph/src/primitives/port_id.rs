use core::ops::Deref;

use crate::primitives::Id;

/// Strongly typed wrapper around the Id primitive for strong type-checking
#[derive(Copy, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PortId(Id);

impl PortId {
    pub const fn new(id: usize) -> Self {
        Self(Id::new(id))
    }

    // TODO: should this number be larger? = 2^8
    // TODO: should this number be dynamic based on the actual
    // number of connections used in the graph at runtime?
    /// This represents the maximum number of connections that a node can have.
    /// Smaller = more memory efficient allocations for the Graph,
    /// since we allocate an array of this size to hold all the necessary connections.
    pub const MAX_PORT_ID: usize = 256;
}

// other convenience implementations possible here

impl Deref for PortId {
    type Target = Id;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I: Into<Id>> From<I> for PortId {
    fn from(value: I) -> Self {
        PortId(value.into())
    }
}
