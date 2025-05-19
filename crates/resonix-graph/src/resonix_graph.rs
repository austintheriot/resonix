use crate::Connectable;

pub trait ResonixGraph {
    fn add(&mut self, connectable: Connectable);
}
