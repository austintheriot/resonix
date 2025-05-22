use crate::DescribePorts;

/// Allows nodes to move ownership of a type implementing `DescribePorts`
/// over to the Graph.
///
/// This makes connecting Node ports after they have already been
/// added to the Graph (the primary user flow) much simpler/ergonomic.
pub trait GetPortDescriptors<P: DescribePorts> {
    fn get_port_descriptors(&self) -> P;
}
