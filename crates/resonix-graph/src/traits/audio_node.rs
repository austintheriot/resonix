use crate::{
    errors::AudioNodeRunError,
    primitives::BlockSize,
    traits::GetNodeId,
    traits::{AudioBuffer, AudioBufferMut},
};

use super::{DescribePorts, GetPriority};

pub trait AudioNode: GetNodeId + GetPriority + DescribePorts {
    fn process<A: AudioBuffer, M: AudioBufferMut>(
        &mut self,
        internal_inputs: &[Option<A>],
        internal_outputs: &mut [Option<M>],
        external_inputs: &[Option<A>],
        external_outputs: &mut [Option<M>],
        block_size: BlockSize,
    ) -> Result<(), AudioNodeRunError>;
}
