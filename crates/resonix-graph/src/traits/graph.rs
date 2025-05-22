use core::ops::Deref;

use crate::{Connectable, ResonixNodeHandle, ResonixPortAddress};

use super::{DescribePorts, GetPorts};

// TODO:fill out with specific types
#[derive(Debug)]
pub struct GraphError;

pub trait ResonixGraph {
    /// `node` must be able to be converted into a `Connectable` and it must deref
    /// to some type that implements `GetPorts`
    fn add<P: DescribePorts, G: GetPorts<P>, N: Into<Connectable> + Deref<Target = G>>(
        &mut self,
        node: N,
    ) -> Result<ResonixNodeHandle<P>, GraphError>;

    fn connect(
        &mut self,
        port_a: ResonixPortAddress,
        port_b: ResonixPortAddress,
    ) -> Result<&mut Self, GraphError>;
}
