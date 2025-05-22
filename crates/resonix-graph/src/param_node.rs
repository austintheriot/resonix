use core::ops::Deref;

use crate::{DescribePorts, GetNodeId};

pub trait ResonixParamNode: DescribePorts + GetNodeId {}

// newtype wrapper due to Rust limitation: https://github.com/rust-lang/rust/issues/20400
pub struct Param<PortDescriptors, A>(pub A)
where
    PortDescriptors: Clone,
    A: ResonixParamNode + Deref<Target = PortDescriptors> + 'static;

impl<PortDescriptors, A> Param<PortDescriptors, A>
where
    PortDescriptors: Clone,
    A: ResonixParamNode + Deref<Target = PortDescriptors> + 'static,
{
    pub fn into_inner(self) -> A {
        self.0
    }
}

impl<PortDescriptors, A> From<A> for Param<PortDescriptors, A>
where
    PortDescriptors: Clone,
    A: ResonixParamNode + Deref<Target = PortDescriptors> + 'static,
{
    fn from(audio_node: A) -> Self {
        Param(audio_node)
    }
}

impl<PortDescriptors, A> Deref for Param<PortDescriptors, A>
where
    PortDescriptors: Clone,
    A: ResonixParamNode + Deref<Target = PortDescriptors> + 'static,
{
    type Target = PortDescriptors;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
