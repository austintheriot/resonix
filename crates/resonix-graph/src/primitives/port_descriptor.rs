use crate::primitives::PortAddress;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PortDescriptor {
    pub address: PortAddress,
    pub channels: usize,
}
