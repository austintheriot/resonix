use common::granular_synthesizer::GranularSynthesizer;

pub trait DensityAction {
    const DEFAULT_DENSITY: f32;

    fn new(gain: f32) -> Self;

    fn get(&self) -> f32;

    fn set(&mut self, density: f32);

    fn sanitize_density(density: f32) -> f32 {
        GranularSynthesizer::sanitize_density(density)
    }
}