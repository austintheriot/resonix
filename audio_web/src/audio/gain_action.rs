pub trait GainAction {
    const GAIN_MIN: f32;
    const GAIN_MAX: f32;

    fn get(&self) -> f32;

    fn set(&mut self, gain: f32);

    fn new(gain: f32) -> Self;

    fn sanitize_gain(input_gain: f32) -> f32 {
        input_gain.max(Self::GAIN_MIN).min(Self::GAIN_MAX)
    }
}
