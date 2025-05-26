use core::ops::Deref;

use crate::primitives::Node;
use crate::primitives::NodeHandle;
use crate::primitives::PortAddress;

use crate::traits::{DescribePorts, GetPortDescriptors};

// TODO: move into the official error module
// TODO: specify error per function?
// TODO:fill out with specific types
#[derive(Debug)]
pub struct GraphError;

pub trait Graph {
    /// `node` must be able to be converted into a `Node` and it must deref
    /// to some type that implements `GetPorts`
    fn add<P: DescribePorts, G: GetPortDescriptors<P>, N: Into<Node> + Deref<Target = G>>(
        &mut self,
        node: N,
    ) -> Result<NodeHandle<P>, GraphError>;

    fn connect(
        &mut self,
        port_a: PortAddress,
        port_b: PortAddress,
    ) -> Result<&mut Self, GraphError>;
}
