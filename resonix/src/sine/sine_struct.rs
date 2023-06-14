use std::f32::consts::PI;

use crate::SampleRate;

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct Sine {
    sample_rate: SampleRate,
    frequency: f32,
    phase: f32,
}

impl Sine {
    pub fn new() -> Self {
        Self {
            sample_rate: SampleRate::new(1u32),
            frequency: 0.0,
            phase: 0.0,
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        let angular_frequency = 2.0 * PI * self.frequency / *self.sample_rate as f32;
        let sample = self.phase.sin();

        // always wrap phase around
        self.phase = (self.phase + angular_frequency) % (2.0 * PI);

        sample
    }

    pub fn next_frame<const N: usize>(&mut self) -> [f32; N] {
        let frame = [self.next_sample(); N];
        frame
    }

    pub fn set_sample_rate(&mut self, sample_rate: impl Into<SampleRate>) -> &mut Self {
        self.sample_rate = sample_rate.into();
        self
    }

    pub fn set_frequency(&mut self, frequency: f32) -> &mut Self {
        self.frequency = frequency;
        self
    }

    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    pub fn frequency(&self) -> f32 {
        self.frequency
    }
}
