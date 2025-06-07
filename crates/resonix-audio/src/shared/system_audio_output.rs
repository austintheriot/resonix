use cpal::Sample;

use crate::SystemAudioOutputError;

pub trait SystemAudioOutput<S: Sample> {
    fn write_sample(&mut self, sample: S) -> Result<(), SystemAudioOutputError>;

    #[cfg(feature = "mock")]
    fn consumer(&mut self) -> Option<Consumer<S>>;
}
