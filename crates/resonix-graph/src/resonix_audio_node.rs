use alloc::vec::Vec;

use crate::{ResonixDataResult, ResonixPortAddress};

pub trait ResonixAudioNode {
    fn next(&mut self) -> ResonixDataResult;

    fn assign_inputs(&mut self, inputs: ResonixDataResult);

    fn input_port_addresses(&self) -> Vec<ResonixPortAddress>;

    fn output_port_addresses(&self) -> Vec<ResonixPortAddress>;
}
