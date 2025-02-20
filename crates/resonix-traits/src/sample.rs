use crate::Amplitude;

pub trait Sample {
    fn from_amplitude(amplitude: impl Amplitude) -> Self;

    fn value(&self) -> impl Amplitude;
}
