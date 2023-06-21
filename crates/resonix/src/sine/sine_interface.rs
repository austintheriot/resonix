use crate::SampleRate;

pub trait SineInterface {
    fn next_sample(&mut self) -> f32;

    fn set_sample_rate(&mut self, sample_rate: impl Into<SampleRate>) -> &mut Self;

    fn set_frequency(&mut self, frequency: f32) -> &mut Self;

    fn sample_rate(&self) -> SampleRate;

    fn frequency(&self) -> f32;
}
