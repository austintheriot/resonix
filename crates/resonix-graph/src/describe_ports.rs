use core::ops::Deref;

use alloc::vec::Vec;

use crate::ResonixPortAddress;

pub trait DescribePorts {
    fn input_port_addresses(&self) -> Vec<ResonixPortAddress>;

    fn output_port_addresses(&self) -> Vec<ResonixPortAddress>;
}

// implement this trait automatically for any traits that Deref
// a struct implementing this trait
impl<D: DescribePorts, T: Deref<Target = D>> DescribePorts for T {
    fn input_port_addresses(&self) -> Vec<ResonixPortAddress> {
        (**self).input_port_addresses()
    }

    fn output_port_addresses(&self) -> Vec<ResonixPortAddress> {
        (**self).input_port_addresses()
    }
}
