use core::ops::Deref;

use crate::{Node, ResonixNodeHandle, ResonixPortAddress};

use super::{DescribePorts, GetPortDescriptors};

// TODO:fill out with specific types
#[derive(Debug)]
pub struct GraphError;

pub trait ResonixGraph {
    /// `node` must be able to be converted into a `Node` and it must deref
    /// to some type that implements `GetPorts`
    fn add<P: DescribePorts, G: GetPortDescriptors<P>, N: Into<Node> + Deref<Target = G>>(
        &mut self,
        node: N,
    ) -> Result<ResonixNodeHandle<P>, GraphError>;

    fn connect(
        &mut self,
        port_a: ResonixPortAddress,
        port_b: ResonixPortAddress,
    ) -> Result<&mut Self, GraphError>;
}
