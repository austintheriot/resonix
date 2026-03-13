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
        inputs: &[Option<A>],
        outputs: &mut [Option<M>],
        block_size: BlockSize,
    ) -> Result<(), AudioNodeRunError>;
}
