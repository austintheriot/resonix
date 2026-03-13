use hashbrown::HashMap;

use crate::errors::GraphAddError;
use crate::errors::GraphConnectionError;
use crate::errors::GraphRunError;
use crate::primitives::ConnectionId;
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

    fn run<A: AudioBuffer, M: AudioBufferMut>(
        &mut self,
        inputs: &HashMap<ConnectionId, A>,
        outputs: &mut HashMap<ConnectionId, M>,
    ) -> Result<(), GraphRunError>;
}
