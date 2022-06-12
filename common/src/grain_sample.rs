/// Associates each Grain's sample value with its envelope value
#[derive(Default)]
pub struct GrainSample {
    /// The value from the buffer that was sampled at the Grain's current_frame.
    pub sample_value: f32,
    /// What volume this grain should be played at.
    pub envelope_value: f32,
}
