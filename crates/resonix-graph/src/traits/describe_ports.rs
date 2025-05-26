use core::ops::Deref;

use alloc::vec::Vec;

use crate::primitives::PortAddress;

/// Having a separate trait/inner object on a node for describing a node's ports
/// allows that functionality to move into the node handle itself
/// once the node has already been moved into the Graph
///
/// This makes connecting Node ports after they have already been
/// added to the Graph (the primary user flow) much simpler/ergonomic.
pub trait DescribePorts {
    fn input_port_addresses(&self) -> Vec<PortAddress>;

    fn output_port_addresses(&self) -> Vec<PortAddress>;
}

// Implement this trait automatically for any traits that Deref
// a struct implementing this trait
//
// Most Nodes would want to Deref to their port descriptors anyway,
// since you want to be able to access that information directly
// from the Node in most cases.
impl<D: DescribePorts, T: Deref<Target = D>> DescribePorts for T {
    fn input_port_addresses(&self) -> Vec<PortAddress> {
        (**self).input_port_addresses()
    }

    fn output_port_addresses(&self) -> Vec<PortAddress> {
        (**self).input_port_addresses()
    }
}
