use crate::{
    errors::AudioNodeRunError,
    primitives::BlockSize,
    traits::{AudioNode, DescribePorts, GetNodeId, GetPriority},
};

/// Internal trait that allows making AudioNode object-safe internally.
///
/// The `AudioNode` trait is not object-safe, but we need `dyn AudioNode` objects for the
/// graph, so this trait wraps the `AudioNode` trait with an internal, concrete
/// implementation to allow using `dyn AudioNode` types in the graph.
///
/// This allows us to just use a concrete `ArrayBuffer` and `ArrayBufferMut`
/// internally, but allows external callers to supply their own implementations as desired.
pub(crate) trait ErasedAudioNode: GetNodeId + GetPriority + DescribePorts {
    fn process(
        &mut self,
        inputs: &[Option<crate::implementations::AudioBuffer<'_>>],
        outputs: &mut [Option<crate::implementations::AudioBufferMut<'_>>],
        block_size: BlockSize,
    ) -> Result<(), AudioNodeRunError>;
}

impl<T: AudioNode> ErasedAudioNode for T {
    fn process(
        &mut self,
        inputs: &[Option<crate::implementations::AudioBuffer<'_>>],
        outputs: &mut [Option<crate::implementations::AudioBufferMut<'_>>],
        block_size: BlockSize,
    ) -> Result<(), AudioNodeRunError> {
        self.process(inputs, outputs, block_size)
    }
}
