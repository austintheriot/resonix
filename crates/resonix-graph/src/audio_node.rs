use alloc::vec::Vec;

use crate::{HasPortDescriptors, ResonixDataList, ResonixId, ResonixPortAddress};

pub trait ResonixAudioNode {
    fn node_id(&self) -> ResonixId;

    fn next(&mut self) -> ResonixDataList;

    fn assign_inputs(&mut self, inputs: ResonixDataList);

    fn input_port_addresses(&self) -> Vec<ResonixPortAddress>;

    fn output_port_addresses(&self) -> Vec<ResonixPortAddress>;
}

// newtype wrapper due to Rust limitation: https://github.com/rust-lang/rust/issues/20400
pub struct Audio<PortDesciptors, A: ResonixAudioNode + HasPortDescriptors<PortDesciptors> + 'static>(
    pub A,
);

impl<PortDescriptors, A: ResonixAudioNode + HasPortDescriptors<PortDescriptors> + 'static> From<A>
    for Audio<PortDescriptors, A>
{
    fn from(value: A) -> Self {
        Audio(value)
    }
}

impl<PortDesciptors, T: ResonixAudioNode + HasPortDescriptors<PortDesciptors> + 'static>
    HasPortDescriptors<PortDesciptors> for Audio<PortDesciptors, T>
{
    fn port_descriptors(&self) -> PortDesciptors {
        self.0.port_descriptors()
    }
}
