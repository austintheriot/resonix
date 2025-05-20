use alloc::vec::Vec;

use crate::ResonixPortAddress;

pub trait ResonixParamNode {
    fn input_port_addresses(&self) -> Vec<ResonixPortAddress>;

    fn output_port_addresses(&self) -> Vec<ResonixPortAddress>;
}
