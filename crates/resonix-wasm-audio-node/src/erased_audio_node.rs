use resonix_graph::{
    errors::AudioNodeRunError,
    primitives::AudioNodeCtx,
    traits::{AudioNode, DescribePorts, GetNodeId, GetPriority},
};

pub(in crate::implementations::graph) trait ErasedAudioNode:
    GetNodeId + GetPriority + DescribePorts
{
    fn process<'buf>(
        &mut self,
        inputs: &[Option<resonix_graph::implementations::OwnedAudioBuffer>],
        outputs: &mut [Option<resonix_graph::implementations::OwnedAudioBuffer>],
        ctx: AudioNodeCtx,
    ) -> Result<(), AudioNodeRunError>;
}

impl<T: AudioNode> ErasedAudioNode for T {
    fn process<'buf>(
        &mut self,
        inputs: &[Option<resonix_graph::implementations::OwnedAudioBuffer>],
        outputs: &mut [Option<resonix_graph::implementations::OwnedAudioBuffer>],
        ctx: AudioNodeCtx,
    ) -> Result<(), AudioNodeRunError> {
        self.process(inputs, outputs, ctx)
    }
}
