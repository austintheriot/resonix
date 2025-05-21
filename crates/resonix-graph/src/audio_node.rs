use core::any::Any;

use alloc::vec::Vec;

use crate::{ResonixDataResult, ResonixPortAddress};

pub trait ResonixAudioNode: Any {
    fn as_any(&self) -> &dyn Any;

    fn next(&mut self) -> ResonixDataResult;

    fn assign_inputs(&mut self, inputs: ResonixDataResult);

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
