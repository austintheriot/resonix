use core::ops::Deref;

use crate::{GetNodeId, ResonixDataList};

use super::GetPriority;

// TODO: remove "Resonix" from all types in this library
pub trait ResonixAudioNode: GetNodeId + GetPriority {
    fn next(&mut self) -> ResonixDataList;

    fn assign_inputs(&mut self, inputs: ResonixDataList);
}

// newtype wrapper due to Rust limitation: https://github.com/rust-lang/rust/issues/20400
// signals to the compiler that the underlying type should be treated as if it ONLY implements
// `ResonixAudioNode` and NOT also `ResonixParamNode`
pub struct Audio<A>(pub A)
where
    A: ResonixAudioNode + 'static;

impl<A> Audio<A>
where
    A: ResonixAudioNode + 'static,
{
    pub fn into_inner(self) -> A {
        self.0
    }
}

/// allows `Audio` to bypass knowing about specific traits
/// the `ResonixAudioNode` might implement for the `Graph`
impl<A> Deref for Audio<A>
where
    A: ResonixAudioNode + 'static,
{
    type Target = A;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
