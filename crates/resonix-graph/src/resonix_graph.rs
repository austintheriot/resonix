use crate::{Connectable, ResonixId, ResonixNodeHandle, ResonixPortHandle};

pub trait ResonixGraph {
    fn add<C: Into<Connectable>>(&mut self, connectable: C) -> ResonixNodeHandle;

    fn connect(port_a: ResonixPortHandle, port_b: ResonixPortHandle) -> Result<(), ()>;

    fn new_id(&mut self) -> ResonixId;
}
