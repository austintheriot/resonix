use std::f32::consts::PI;

use crate::{SampleRate, SineInterface};

const ZERO_SAMPLE_RATE: SampleRate = SampleRate::new_const(0);

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

impl SineInterface for Sine {
    fn next_sample(&mut self) -> f32 {
        if self.sample_rate == ZERO_SAMPLE_RATE {
            return 0.0;
        }

        let sample = self.phase.sin();

        self.phase += self.angular_frequency;

        // always wrap phase around to be within unit circle
        self.phase %= TWO_PI;

        sample
    }

    fn set_sample_rate(&mut self, sample_rate: impl Into<SampleRate>) -> &mut Self {
        let prev_sample_rate = self.sample_rate;
        let sample_rate = sample_rate.into();

        // only run calculations if necessary
        if prev_sample_rate != sample_rate {
            self.sample_rate = sample_rate;
            self.angular_frequency = self.calculate_own_angular_frequency();
        }

        self
    }

    fn set_frequency(&mut self, frequency: f32) -> &mut Self {
        let prev_frequency = self.frequency;

        // only run calculations if necessary
        if prev_frequency != frequency {
            self.frequency = frequency;
            self.angular_frequency = self.calculate_own_angular_frequency();
        }

        self
    }

    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    fn frequency(&self) -> f32 {
        self.frequency
    }
}

impl Sine {
    pub fn new() -> Self {
        Self {
            sample_rate: SampleRate::new(1u32),
            frequency: 0.0,
            phase: 0.0,
            angular_frequency: 0.0,
        }
    }

    pub fn new_with_config(sample_rate: impl Into<SampleRate>, frequency: impl Into<f32>) -> Self {
        let frequency = frequency.into();
        let sample_rate = sample_rate.into();
        Sine {
            sample_rate,
            frequency,
            phase: 0.0,
            angular_frequency: Self::calculate_angular_frequency(frequency, *sample_rate as f32),
        }
    }

    fn calculate_own_angular_frequency(&self) -> f32 {
        Self::calculate_angular_frequency(self.frequency, *self.sample_rate as f32)
    }

    fn calculate_angular_frequency(frequency: f32, sample_rate: f32) -> f32 {
        if sample_rate == 0.0 {
            return 0.0;
        }

        TWO_PI * frequency / sample_rate
    }
}

#[cfg(test)]
mod test_sine {
    use resonix_test_utils::assert_difference_is_within_tolerance;

    use crate::{Sine, SineInterface};

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

    #[test]
    fn it_should_work_when_sample_rate_is_0() {
        const SAMPLE_RATE: u32 = 0;
        let mut sine = Sine::new();
        sine.set_frequency(1.0).set_sample_rate(SAMPLE_RATE);

        for _ in 0..5 {
            sine.next_sample();
        }

        assert_eq!(sine.next_sample(), 0.0);
        assert_eq!(sine.phase, 0.0);
        assert_eq!(sine.angular_frequency, 0.0)
    }
}
