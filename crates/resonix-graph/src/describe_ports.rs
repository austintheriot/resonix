use core::ops::Deref;

use alloc::vec::Vec;

use crate::ResonixPortAddress;

pub trait DescribePorts {
    fn input_port_addresses(&self) -> Vec<ResonixPortAddress>;

    fn output_port_addresses(&self) -> Vec<ResonixPortAddress>;
}

impl<D: DescribePorts, T: Deref<Target = D>> DescribePorts for T {
    fn input_port_addresses(&self) -> Vec<ResonixPortAddress> {
        (**self).input_port_addresses()
    }

    fn output_port_addresses(&self) -> Vec<ResonixPortAddress> {
        (**self).input_port_addresses()
    }
}
