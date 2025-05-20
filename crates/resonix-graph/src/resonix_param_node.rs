use alloc::vec::Vec;

use crate::ResonixPortAddress;

pub trait ResonixParamNode {
    fn port_addresses(&self) -> Vec<ResonixPortAddress>;
}
