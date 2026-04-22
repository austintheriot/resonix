use resonix_core::{
    errors::AudioNodeRunError,
    primitives::AudioNodeCtx,
    traits::{AudioNode, DescribePorts, GetNodeId, GetPriority},
};

pub trait ErasedAudioNode: GetNodeId + GetPriority + DescribePorts {
    fn process<'buf>(
        &mut self,
        inputs: &[Option<resonix_core::implementations::OwnedAudioBuffer>],
        outputs: &mut [Option<resonix_core::implementations::OwnedAudioBuffer>],
        ctx: AudioNodeCtx,
    ) -> Result<(), AudioNodeRunError>;
}

impl<T: AudioNode> ErasedAudioNode for T {
    fn process<'buf>(
        &mut self,
        inputs: &[Option<resonix_core::implementations::OwnedAudioBuffer>],
        outputs: &mut [Option<resonix_core::implementations::OwnedAudioBuffer>],
        ctx: AudioNodeCtx,
    ) -> Result<(), AudioNodeRunError> {
        self.process(inputs, outputs, ctx)
    }
}
