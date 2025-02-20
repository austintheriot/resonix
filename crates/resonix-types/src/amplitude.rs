pub trait Amplitude {
    fn from_f64(amplitude: f64) -> Self;

    fn as_f64(&self) -> f64;
}
