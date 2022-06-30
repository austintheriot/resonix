use super::density_action::DensityAction;
use audio_common::{
    granular_synthesizer::GranularSynthesizer,
    granular_synthesizer_action::GranularSynthesizerAction, percentage::Percentage,
};

/// Wrapper around raw `f32` value for access on the audio thread
#[derive(Clone, Copy, Debug)]
pub struct Density(Percentage);

impl DensityAction for Density {
    const DEFAULT_DENSITY: f32 = GranularSynthesizer::DEFAULT_DENSITY;

    fn new(gain: impl Into<Percentage>) -> Self {
        Self(gain.into())
    }

    fn get(&self) -> Percentage {
        self.0
    }

    fn set(&mut self, density: impl Into<Percentage>) {
        self.0 = density.into();
    }
}

impl Default for Density {
    fn default() -> Self {
        Self(Self::DEFAULT_DENSITY.into())
    }
}
