use cpal::Sample;

use crate::SystemAudioOutputError;

pub trait SystemAudioOutput<S: Sample> {
    fn write_sample(&mut self, sample: S) -> Result<(), SystemAudioOutputError>;
}
