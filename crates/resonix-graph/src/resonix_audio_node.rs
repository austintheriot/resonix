use alloc::vec::Vec;

use crate::{ResonixDataResult, ResonixPortAddress};

pub trait ResonixAudioNode {
    fn next(&mut self) -> ResonixDataResult;

    fn assign_inputs(&mut self, inputs: ResonixDataResult);

    fn port_addresses(&self) -> Vec<ResonixPortAddress>;
}
