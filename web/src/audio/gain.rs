/// Wrapper around raw `f32` value for access on the audio thread
#[derive(Clone, Copy, Debug)]
pub struct Gain(pub f32);

impl Default for Gain {
    fn default() -> Self {
        Self(1.0)
    }
}

impl Gain {
    pub fn new(gain: f32) -> Self {
        Self(gain)
    }
}