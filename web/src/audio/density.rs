use common::granular_synthesizer::GranularSynthesizer;

/// Wrapper around raw `f32` value for access on the audio thread
#[derive(Clone, Copy, Debug)]
pub struct Density(pub f32);

impl Default for Density {
    fn default() -> Self {
        Self(GranularSynthesizer::DEFAULT_DENSITY)
    }
}

impl Density {
    pub fn new(gain: f32) -> Self {
        Self(gain)
    }
}