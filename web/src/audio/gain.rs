use crate::audio::gain_action::GainAction;

/// Wrapper around raw `f32` value for access on the audio thread
#[derive(Clone, Copy, Debug)]
pub struct Gain(f32);

impl GainAction for Gain {
    const GAIN_MIN: f32 = 0.0;
    const GAIN_MAX: f32 = 1.0;

    fn get(&self) -> f32 {
        self.0
    }

    fn set(&mut self, gain: f32) {
        self.0 = Self::sanitize_gain(gain);
    }

    fn new(gain: f32) -> Self {
        Self(Self::sanitize_gain(gain))
    }
}

impl Default for Gain {
    fn default() -> Self {
        Self(Self::GAIN_MAX)
    }
}