use crate::{
    errors::AudioNodeRunError,
    primitives::AudioNodeCtx,
    traits::{AudioBuffer, AudioBufferMut, GetNodeId},
};

use super::{DescribePorts, GetPriority};

pub trait AudioNode: GetNodeId + GetPriority + DescribePorts {
    /// Runs an AudioNode's audio processing logic.
    ///
    /// `inputs` and `outputs` as indexed by the node's `PortId`.
    /// `PortId`s are expected to be densely mapped,
    /// and the passed in buffers are derived based on
    /// the `PortId` the mapping provided by the node's
    /// `PortDescriptor`s
    fn process<A: AudioBuffer, M: AudioBufferMut>(
        &mut self,
        inputs: &[Option<A>],
        outputs: &mut [Option<M>],
        ctx: AudioNodeCtx,
    ) -> Result<(), AudioNodeRunError>;
}
