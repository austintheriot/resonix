use crate::audio::gain::Gain;

pub trait GainAction {
    fn sanitize_gain(input_gain: f32) -> f32 {
        input_gain.max(Gain::GAIN_MIN).min(Gain::GAIN_MAX)
    }

    fn get(&self) -> f32;

    fn set(&mut self, gain: f32);

    fn new(gain: f32) -> Self;
}