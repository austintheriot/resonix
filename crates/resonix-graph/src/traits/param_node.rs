use core::ops::Deref;

use crate::traits::GetNodeId;

use super::GetPriority;

pub trait ParamNode: GetNodeId + GetPriority {}

// newtype wrapper due to Rust limitation: https://github.com/rust-lang/rust/issues/20400
// signals to the compiler that the underlying type should be treated as if it ONLY implements
// `ParamNode` and NOT also `AudioNode`
pub struct Param<P>(pub P)
where
    P: ParamNode + 'static;

impl<A> Param<A>
where
    A: ParamNode + 'static,
{
    pub fn into_inner(self) -> A {
        self.0
    }
}

/// allows `Param` to bypass knowing about specific traits
/// the `ParamNode` might implement for the `Graph`
impl<P> Deref for Param<P>
where
    P: ParamNode + 'static,
{
    type Target = P;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
