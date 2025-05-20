use crate::{Connectable, ResonixNodeHandle, ResonixPortAddress};

pub trait ResonixGraph {
    fn add<C: Into<Connectable>>(&mut self, connectable: C) -> ResonixNodeHandle;

    fn connect(&mut self, port_a: ResonixPortAddress, port_b: ResonixPortAddress)
    -> Result<(), ()>;
}
