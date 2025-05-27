use core::{error::Error, ops::Deref};

use alloc::boxed::Box;

use hashbrown::HashMap;

use crate::{
    errors::AudioNodeAssignInputError,
    primitives::{DataList, PortAddress},
    traits::GetNodeId,
};

use super::GetPriority;

pub trait AudioNode: GetNodeId + GetPriority {
    fn run(&mut self) -> Result<Option<HashMap<PortAddress, DataList>>, Box<dyn Error>>;

    fn assign_inputs(&mut self, inputs: DataList) -> Result<(), AudioNodeAssignInputError>;
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
