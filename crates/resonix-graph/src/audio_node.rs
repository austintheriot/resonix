use core::ops::Deref;

use crate::{DescribePorts, ResonixDataList, ResonixId, ResonixPortAddress};

pub trait ResonixAudioNode: DescribePorts {
    fn node_id(&self) -> ResonixId;

    fn next(&mut self) -> ResonixDataList;

    fn assign_inputs(&mut self, inputs: ResonixDataList);
}

// newtype wrapper due to Rust limitation: https://github.com/rust-lang/rust/issues/20400
pub struct Audio<PortDescriptors, A>(pub A)
where
    PortDescriptors: Clone,
    A: ResonixAudioNode + Deref<Target = PortDescriptors> + 'static;

impl<PortDescriptors, A> Audio<PortDescriptors, A>
where
    PortDescriptors: Clone,
    A: ResonixAudioNode + Deref<Target = PortDescriptors> + 'static,
{
    pub fn into_inner(self) -> A {
        self.0
    }
}

impl<PortDescriptors, A> From<A> for Audio<PortDescriptors, A>
where
    PortDescriptors: Clone,
    A: ResonixAudioNode + Deref<Target = PortDescriptors> + 'static,
{
    fn from(audio_node: A) -> Self {
        Audio(audio_node)
    }
}

impl<PortDescriptors, A> Deref for Audio<PortDescriptors, A>
where
    PortDescriptors: Clone,
    A: ResonixAudioNode + Deref<Target = PortDescriptors> + 'static,
{
    type Target = PortDescriptors;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
