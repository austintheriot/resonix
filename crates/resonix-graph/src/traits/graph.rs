use crate::{
    errors::GraphRunError,
    primitives::AudioNodeCtx,
    traits::{AudioBuffer, AudioBufferMut, ModifyGraph},
};

/// Immediate-mode graph.
///
/// Caller provides input/output data at call time.
pub trait Graph: ModifyGraph {
    /// TODO: represent this in the type system
    /// `inputs` and `outputs` are indexed by `ExternalConnectionId`
    fn run<A: AudioBuffer, M: AudioBufferMut>(
        &mut self,
        inputs: &[Option<A>],
        outputs: &mut [Option<M>],
        ctx: AudioNodeCtx,
    ) -> Result<(), GraphRunError>;
}
