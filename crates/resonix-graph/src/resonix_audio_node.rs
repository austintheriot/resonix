use crate::{ResonixDataResult, ResonixId};

pub trait ResonixAudioNode {
    fn next(&mut self) -> ResonixDataResult;

    fn node_id(&self) -> ResonixId;

    fn assign_inputs(&mut self, inputs: ResonixDataResult);
}
