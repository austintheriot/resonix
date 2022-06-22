use crate::audio::gain_action::GainAction;

/// Wrapper around raw `f32` value for access on the audio thread
#[derive(Clone, Copy, Debug)]
pub struct Gain(pub f32);

impl Default for Gain {
    fn default() -> Self {
        Self(1.0)
    }
}

impl Gain {
    pub const GAIN_MIN: f32 = 0.0;
    pub const GAIN_MAX: f32 = 1.0;
}

impl GainAction for Gain {
    fn get(&self) -> f32 {
        self.0
    }

    fn set(&mut self, gain: f32) {
        self.0 = Gain::sanitize_gain(gain);
    }

    fn new(gain: f32) -> Self {
        Self(Gain::sanitize_gain(gain))
    }
}