use crate::{Connectable, ResonixNodeHandle, ResonixPortAddress};

// TODO:fill out with specific types
#[derive(Debug)]
pub struct GraphError;

pub trait ResonixGraph {
    fn add<C: Into<Connectable>>(&mut self, connectable: C) -> ResonixNodeHandle;

    fn connect(
        &mut self,
        port_a: ResonixPortAddress,
        port_b: ResonixPortAddress,
    ) -> Result<(), GraphError>;
}
