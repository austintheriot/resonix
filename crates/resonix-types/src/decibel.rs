use crate::Amplitude;

pub trait Decibel {
    fn calculate(reference_amplitude: impl Amplitude, amplitude: impl Amplitude) -> impl Decibel;
}
