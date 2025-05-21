use alloc::vec::Vec;

use crate::{ResonixDataList, ResonixId, ResonixPortAddress};

pub trait ResonixAudioNode {
    fn node_id(&self) -> ResonixId;

    fn next(&mut self) -> ResonixDataList;

    fn assign_inputs(&mut self, inputs: ResonixDataList);

    fn input_port_addresses(&self) -> Vec<ResonixPortAddress>;

    fn output_port_addresses(&self) -> Vec<ResonixPortAddress>;
}

impl<R: ResonixAudioNode + 'static> From<R> for Audio<R> {
    fn from(value: R) -> Self {
        Audio(value)
    }
}

// newtype wrapper due to Rust limitation: https://github.com/rust-lang/rust/issues/20400
pub struct Audio<T: ResonixAudioNode + 'static>(pub T);
