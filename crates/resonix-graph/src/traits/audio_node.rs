use core::{any::Any, ops::Deref};

use hashbrown::HashMap;

use crate::{
    errors::AudioNodeRunError,
    primitives::{Data, PortAddress},
    traits::GetNodeId,
};

use super::{DescribePorts, GetPriority};

// TODO: remove `Any` type rescription--we can use port descriptors to accomplish this
pub trait AudioNode: GetNodeId + GetPriority + DescribePorts + Any {
    // TODO: adjust signature to pass an array of mutable pointers, so the node doesn't need to
    // directly access a hashmap
    fn process(
        &mut self,
        inputs: &HashMap<PortAddress, &Data>,
    ) -> Result<Option<HashMap<PortAddress, Data>>, AudioNodeRunError>;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
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
