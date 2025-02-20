use crate::Amplitude;

pub struct Sample<A: resonix_types::Amplitude = Amplitude> {
    amplitude: A,
}

impl<A: resonix_types::Amplitude> resonix_types::Sample for Sample {
    fn from_amplitude(amplitude: A) -> Self {
        Self { amplitude }
    }

    fn value(&self) -> &A {
        &self.amplitude
    }

    fn into_value(self) -> A {
        self.amplitude
    }
}
