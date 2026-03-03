use core::ops::Deref;

use hashbrown::HashMap;

use crate::errors::GraphAddError;
use crate::errors::GraphConnectionError;
use crate::errors::GraphRunError;
use crate::primitives::ConnectionId;
use crate::primitives::Node;
use crate::primitives::NodeHandle;
use crate::primitives::PortAddress;

use crate::primitives::Sample;
use crate::traits::{DescribePorts, GetPortDescriptors};

pub trait Graph {
    /// `node` must be able to be converted into a `Node` and it must deref
    /// to some type that implements `GetPorts`
    fn add<P: DescribePorts, G: GetPortDescriptors<P>, N: Into<Node> + Deref<Target = G>>(
        &mut self,
        node: N,
    ) -> Result<NodeHandle<P>, GraphAddError>;

    fn connect(
        &mut self,
        port_a: PortAddress,
        port_b: PortAddress,
    ) -> Result<&mut Self, GraphConnectionError>;

    fn run(
        &mut self,
        inputs: &HashMap<ConnectionId, &[Sample]>,
        outputs: &mut HashMap<ConnectionId, &mut [Sample]>,
    ) -> Result<(), GraphRunError>;
}
