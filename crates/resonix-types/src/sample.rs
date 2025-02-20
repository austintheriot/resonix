use crate::Amplitude;

pub trait Sample<A: Amplitude> {
    fn from_amplitude(amplitude: A) -> Self;

    fn value(&self) -> &A;

    fn into_value(self) -> A;
}
