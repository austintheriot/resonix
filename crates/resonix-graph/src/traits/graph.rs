use crate::errors::GraphAddError;
use crate::errors::GraphConnectionError;
use crate::errors::GraphRunError;
use crate::primitives::CurrentTime;
use crate::primitives::NodeHandle;
use crate::primitives::PortAddress;
use crate::traits::AudioBuffer;
use crate::traits::AudioBufferMut;

use crate::traits::{DescribePorts, GetPortDescriptors};

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
        current_time: CurrentTime,
    ) -> Result<(), GraphRunError>;
}
