use core::ops::Deref;

use crate::{Connectable, ResonixNodeHandle, ResonixPortAddress};

use super::GetPorts;

// TODO:fill out with specific types
#[derive(Debug)]
pub struct GraphError;

pub trait ResonixGraph {
    fn add<G: GetPorts<Ports>, Ports, C: Into<Connectable> + Deref<Target = G>>(
        &mut self,
        connectable: C,
    ) -> Result<ResonixNodeHandle<Ports>, GraphError>;

    fn connect(
        &mut self,
        port_a: ResonixPortAddress,
        port_b: ResonixPortAddress,
    ) -> Result<&mut Self, GraphError>;
}
