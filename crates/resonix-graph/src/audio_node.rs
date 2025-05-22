use core::ops::Deref;

use alloc::vec::Vec;

use crate::{DescribePorts, HasPortDescriptors, ResonixDataList, ResonixId, ResonixPortAddress};

pub trait ResonixAudioNode: DescribePorts {
    fn node_id(&self) -> ResonixId;

    fn next(&mut self) -> ResonixDataList;

    fn assign_inputs(&mut self, inputs: ResonixDataList);
}


// newtype wrapper due to Rust limitation: https://github.com/rust-lang/rust/issues/20400
pub struct Audio<PortDescriptors, A>
where
    PortDescriptors: Clone,
    A: ResonixAudioNode + HasPortDescriptors<PortDescriptors = PortDescriptors> + 'static,
{
    audio_node: A,
    port_descriptors: PortDescriptors,
}

impl<PortDescriptors, A> Audio<PortDescriptors, A>
where
    PortDescriptors: Clone,
    A: ResonixAudioNode + HasPortDescriptors<PortDescriptors = PortDescriptors> + 'static,
{
    pub fn into_inner(self) -> A {
        self.audio_node
    }
}

impl<PortDescriptors, A> From<A> for Audio<PortDescriptors, A>
where
    PortDescriptors: Clone,
    A: ResonixAudioNode + HasPortDescriptors<PortDescriptors = PortDescriptors> + 'static,
{
    fn from(audio_node: A) -> Self {
        let port_descriptors = audio_node.port_descriptors();
        Audio {
            audio_node,
            port_descriptors,
        }
    }
}

impl<PortDescriptors, A> AsRef<PortDescriptors> for Audio<PortDescriptors, A>
where
    PortDescriptors: Clone,
    A: ResonixAudioNode + HasPortDescriptors<PortDescriptors = PortDescriptors> + 'static,
{
    fn as_ref(&self) -> &PortDescriptors {
        &self.port_descriptors
    }
}
