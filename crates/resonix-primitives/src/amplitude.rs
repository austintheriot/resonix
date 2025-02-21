pub struct Amplitude(f64);

impl resonix_types::Amplitude for Amplitude {
    fn from_f64(amplitude: f64) -> Self {
        Self(amplitude)
    }

    fn as_f64(&self) -> f64 {
        self.0
    }
}
