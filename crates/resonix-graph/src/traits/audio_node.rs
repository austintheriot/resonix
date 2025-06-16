use core::ops::Deref;

use crate::{
    errors::AudioNodeRunError,
    primitives::{Data, DataBlock},
    traits::GetNodeId,
};

use super::{DescribePorts, GetPriority};

pub trait AudioNode: GetNodeId + GetPriority + DescribePorts {
    fn process(
        &mut self,
        inputs: &[&DataBlock],
        // If a Node connects to itself, only its output block will be supplied
        // since the data can't be accessed both mutably and immutably at the same time
        outputs: &mut [&mut DataBlock],
    ) -> Result<(), AudioNodeRunError>;
}

// newtype wrapper due to Rust limitation: https://github.com/rust-lang/rust/issues/20400
// signals to the compiler that the underlying type should be treated as if it ONLY implements
// `AudioNode` and NOT also `ParamNode`
pub struct Audio<A>(pub A)
where
    A: AudioNode + 'static;

impl<A> Audio<A>
where
    A: AudioNode + 'static,
{
    pub fn into_inner(self) -> A {
        self.0
    }
}

/// allows `Audio` to bypass knowing about specific traits
/// the `AudioNode` might implement for the `Graph`
impl<A> Deref for Audio<A>
where
    A: AudioNode + 'static,
{
    type Target = A;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
