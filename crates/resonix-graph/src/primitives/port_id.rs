use core::ops::Deref;

use crate::ResonixId;

/// Strongly typed wrapper around the Id primitive for strong type-checking
#[derive(Copy, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PortId(ResonixId);

impl PortId {
    pub const fn new(id: usize) -> Self {
        Self(ResonixId::new(id))
    }
}

// other convenience implementations possible here

impl Deref for PortId {
    type Target = ResonixId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I: Into<ResonixId>> From<I> for PortId {
    fn from(value: I) -> Self {
        PortId(value.into())
    }
}
