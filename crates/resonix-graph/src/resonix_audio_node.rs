use crate::ResonixDataResult;

pub trait ResonixAudioNode {
    fn next(&mut self) -> ResonixDataResult;

    fn assign_inputs(&mut self, inputs: ResonixDataResult);
}
