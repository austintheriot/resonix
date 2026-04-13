use crate::{
    errors::{GraphAddError, GraphConnectionError, GraphRunError},
    primitives::{AudioNodeCtx, BlockSize, ExternalBufferMappings, NodeHandle, PortAddress},
    traits::{AudioBuffer, AudioBufferMut, DescribePorts, GetPortDescriptors},
};

use super::AudioNode;

pub trait Graph {
    fn add_audio_node<P: DescribePorts, N: AudioNode + GetPortDescriptors<P> + 'static>(
        &mut self,
        node: N,
    ) -> Result<NodeHandle<P>, GraphAddError>;

    fn connect(
        &mut self,
        port_a: PortAddress,
        port_b: PortAddress,
    ) -> Result<&mut Self, GraphConnectionError>;

    /// TODO: represent this in the type system
    /// `inputs` and `outputs` are indexed by `ExternalConnectionId`
    fn run<A: AudioBuffer, M: AudioBufferMut>(
        &mut self,
        inputs: &[Option<A>],
        outputs: &mut [Option<M>],
        ctx: AudioNodeCtx,
    ) -> Result<(), GraphRunError>;

    fn block_size(&self) -> BlockSize;

    // TODO: remove `mut` ?
    fn external_buffer_mappings(&mut self) -> ExternalBufferMappings;

    /// Convenience method: provides a good caller-specified time for the Graph
    /// to do any sort of heavy, allocation work that otherwise should
    /// not be done in the audio hot loop.
    ///
    /// Signals to the Graph that the caller is done making edits to the
    /// audio graph for the time being.
    ///
    /// Is not guaranteed to be called by the actual wrapping context of
    /// a `Graph`.
    fn compile(&mut self) {}
}
