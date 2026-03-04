use core::ops::Deref;

use crate::primitives::PortDescriptor;

/// Having a separate trait/inner object on a node for describing a node's ports
/// allows that functionality to move into the node handle itself
/// once the node has already been moved into the Graph
///
/// This makes connecting Node ports after they have already been
/// added to the Graph (the primary user flow) much simpler/ergonomic.
pub trait DescribePorts {
    fn input_ports(&self) -> Option<&[PortDescriptor]> {
        None
    }

    fn output_ports(&self) -> Option<&[PortDescriptor]> {
        None
    }

    fn external_output_ports(&self) -> Option<&[PortDescriptor]> {
        None
    }

    fn external_input_ports(&self) -> Option<&[PortDescriptor]> {
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

    fn external_output_ports(&self) -> Option<&[PortDescriptor]> {
        (**self).external_output_ports()
    }

    fn external_input_ports(&self) -> Option<&[PortDescriptor]> {
        (**self).external_input_ports()
    }
}
