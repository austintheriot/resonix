/// Returns a single sample, representing the audio data
/// that will be output to the DAC
pub trait OutputNode {
    fn data(&self) -> f32;
}