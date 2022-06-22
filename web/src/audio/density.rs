use super::density_action::DensityAction;
use common::granular_synthesizer::GranularSynthesizer;

/// Wrapper around raw `f32` value for access on the audio thread
#[derive(Clone, Copy, Debug)]
pub struct Density(f32);

impl DensityAction for Density {
    const DEFAULT_DENSITY: f32 = GranularSynthesizer::DEFAULT_DENSITY;

    fn new(gain: f32) -> Self {
        Self(Self::sanitize_density(gain))
    }

    fn get(&self) -> f32 {
        self.0
    }

    fn set(&mut self, density: f32) {
        self.0 = Self::sanitize_density(density);
    }
}

impl Default for Density {
    fn default() -> Self {
        Self(Self::DEFAULT_DENSITY)
    }
}
