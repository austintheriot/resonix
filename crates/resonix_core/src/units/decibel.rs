use std::ops::{Div, Mul};

/// allows the maximum amplitude of (1.0) to be 100 dB,
/// minimum amplitude of (0.0) to be negative infinity,
/// and 0 dB to be inaudibly quiet at any reasonable listening level
pub const DECIBEL_DEFAULT_REFERENCE_AMPLITUDE: f32 = 0.00001;

/// decibel (dB) level is defined as: `d = 20 · log10(a/a0)`
/// where a0 is a given reference amplitude.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Decibel {
    reference_amplitude: f32,
    amplitude: f32,
}

impl Decibel {
    pub fn get(&self) -> f32 {
        Self::calculate(self.reference_amplitude, self.amplitude)
    }

    pub fn calculate(reference_amplitude: f32, amplitude: f32) -> f32 {
        amplitude.div(reference_amplitude).log10().mul(20.0)
    }

    pub fn calculate_with_default_reference(amplitude: f32) -> f32 {
        Self::calculate(DECIBEL_DEFAULT_REFERENCE_AMPLITUDE, amplitude)
    }
}

impl Default for Decibel {
    fn default() -> Self {
        Self {
            reference_amplitude: DECIBEL_DEFAULT_REFERENCE_AMPLITUDE,
            amplitude: 0.0,
        }
    }
}

#[cfg(test)]
mod test_decibel {
    use crate::Decibel;

    #[test]
    pub fn it_should_return_neg_inf_for_amplitude_0() {
        let result = Decibel::calculate_with_default_reference(0.0);
        assert_eq!(result, f32::NEG_INFINITY);
    }

    #[test]
    pub fn it_should_return_100_for_amplitude_1() {
        let result = Decibel::calculate_with_default_reference(1.0);
        assert_eq!(result, 100.0);
    }
}
