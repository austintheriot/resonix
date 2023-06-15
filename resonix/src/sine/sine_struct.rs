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
        [self.next_sample(); N]
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

#[cfg(test)]
mod test_sine {
    use crate::Sine;

    #[track_caller]
    fn assert_difference_is_within_tolerance(value: f32, expected: f32, tolerance: f32) {
        let difference_from_expected_amplitude = f32::abs((expected) - (value));
        assert!((difference_from_expected_amplitude) < (tolerance));
    }

    #[test]
    fn it_should_produce_sine_values_at_given_frequency() {
        const SAMPLE_RATE: u32 = 16;
        const ASSERTION_TOLERANCE: f32 = 0.00001;
        let mut sine = Sine::new();
        sine.set_frequency(1.0).set_sample_rate(SAMPLE_RATE);

        // first value is 0.0
        assert_difference_is_within_tolerance(sine.next_sample(), 0.0, ASSERTION_TOLERANCE);

        for _ in 0..(SAMPLE_RATE / 4 - 1) {
            sine.next_sample();
        }

        // after iterating 1/4 way through wave length,
        // the value should be 1.0
        assert_difference_is_within_tolerance(sine.next_sample(), 1.0, ASSERTION_TOLERANCE);

        for _ in 0..(SAMPLE_RATE / 4 - 1) {
            sine.next_sample();
        }

        // after iterating 1/2 way through wave length,
        // the value should be close 0.0
        assert_difference_is_within_tolerance(sine.next_sample(), 0.0, ASSERTION_TOLERANCE);

        for _ in 0..(SAMPLE_RATE / 4 - 1) {
            sine.next_sample();
        }

        // after iterating 3/4 way through wave length,
        // the value should be -1.0
        assert_difference_is_within_tolerance(sine.next_sample(), -1.0, ASSERTION_TOLERANCE);

        for _ in 0..(SAMPLE_RATE / 4 - 1) {
            sine.next_sample();
        }

        // after iterating all the way through wave length,
        // the value should be 0.0
        assert_difference_is_within_tolerance(sine.next_sample(), 0.0, ASSERTION_TOLERANCE);
    }
}
