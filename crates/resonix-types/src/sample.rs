use crate::Amplitude;

pub trait Sample {
    fn from_amplitude(amplitude: impl Amplitude) -> impl Sample;

    fn value(&self) -> impl Amplitude;
}
