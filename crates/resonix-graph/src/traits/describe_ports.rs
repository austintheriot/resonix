use core::ops::Deref;

use crate::primitives::PortDescriptor;

/// Having a separate trait/inner object on a node for describing a node's ports
/// allows that functionality to move into the node handle itself
/// once the node has already been moved into the Graph
///
/// This makes connecting Node ports after they have already been
/// added to the Graph (the primary user flow) much simpler/ergonomic.
///
/// # PortId invariants
///
/// All ports within the **input** direction — regardless of whether they are
/// internal (pool-backed, `PortAddressDirection::Input`) or external
/// (caller-supplied, `PortAddressDirection::ExternalInput`) — must share a
/// single, dense, monotonically increasing `PortId` namespace starting at 0.
/// Concretely, if a node has N input ports their PortIds must be exactly 0..N.
///
/// The same rule applies independently to the **output** direction: all output
/// ports (internal `Output` and external `ExternalOutput`) must use PortIds 0..M
/// where M is the total number of output ports.
///
/// The graph uses these PortIds as direct slice indices, so gaps or duplicates
/// will cause incorrect behaviour or a `GraphAddError::SparsePortIds` at add-time.
/// Whether a port is internal or external is determined at call sites by
/// inspecting `PortDescriptor::address.port_address_direction()`.
pub trait DescribePorts {
    fn input_ports(&self) -> Option<&[PortDescriptor]> {
        None
    }

    fn output_ports(&self) -> Option<&[PortDescriptor]> {
        None
    }
}

// Implement this trait automatically for any traits that Deref
// a struct implementing this trait
//
// Most Nodes would want to Deref to their port descriptors anyway,
// since you want to be able to access that information directly
// from the Node in most cases.
impl<D: DescribePorts + ?Sized, T: Deref<Target = D>> DescribePorts for T
where
    for<'x> D: 'x,
{
    fn input_ports(&self) -> Option<&[PortDescriptor]> {
        (**self).input_ports()
    }

    fn output_ports(&self) -> Option<&[PortDescriptor]> {
        (**self).output_ports()
    }
}
