use std::f32::consts::PI;

use crate::SampleRate;

/// Produces a sine wave at the given frequency and sample rate
///
/// e.g. at a sample_rate of `100` and a frequency of `1.0`,
/// `Sine` will complete a full wavelength cycle after `next_sample`
/// has been called 100 times.
///
/// At a sample_rate of `44100` and a frequency of `440.0`, the
/// note `A` will be produced, so long as the sample_rate
/// given to sine matches the audio context's sample rate.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct Sine {
    sample_rate: SampleRate,
    frequency: f32,
    phase: f32,
    angular_frequency: f32,
}

const TWO_PI: f32 = 2.0 * PI;

impl Sine {
    pub fn new() -> Self {
        Self {
            sample_rate: SampleRate::new(1u32),
            frequency: 0.0,
            phase: 0.0,
            angular_frequency: 0.0,
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        let sample = self.phase.sin();

        self.phase += self.angular_frequency;

        // always wrap phase around to be within unit circle
        self.phase %= TWO_PI;

        sample
    }

    pub fn next_frame<const N: usize>(&mut self) -> [f32; N] {
        [self.next_sample(); N]
    }

    pub fn set_sample_rate(&mut self, sample_rate: impl Into<SampleRate>) -> &mut Self {
        let prev_sample_rate = self.sample_rate;
        let sample_rate = sample_rate.into();

        // only run calculations if necessary
        if prev_sample_rate != sample_rate {
            self.sample_rate = sample_rate;
            self.angular_frequency = self.calculate_angular_frequency();
        }

        self
    }

    pub fn set_frequency(&mut self, frequency: f32) -> &mut Self {
        let prev_frequency = self.frequency;

        // only run calculations if necessary
        if prev_frequency != frequency {
            self.frequency = frequency;
            self.angular_frequency = self.calculate_angular_frequency();
        }

        self
    }

    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    pub fn frequency(&self) -> f32 {
        self.frequency
    }

    fn calculate_angular_frequency(&self) -> f32 {
        TWO_PI * self.frequency / *self.sample_rate as f32
    }
}

#[cfg(test)]
mod test_sine {
    use resonix_test_utils::assert_difference_is_within_tolerance;

    use crate::Sine;

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
