use core::ops::Deref;

use crate::{Connectable, ResonixNodeHandle, ResonixPortAddress};

// TODO:fill out with specific types
#[derive(Debug)]
pub struct GraphError;

pub trait ResonixGraph {
    fn add<PortDescriptors: Clone, C: Into<Connectable> + Deref<Target = PortDescriptors>>(
        &mut self,
        connectable: C,
    ) -> ResonixNodeHandle<PortDescriptors>;

    fn connect(
        &mut self,
        port_a: ResonixPortAddress,
        port_b: ResonixPortAddress,
    ) -> Result<(), GraphError>;
}
