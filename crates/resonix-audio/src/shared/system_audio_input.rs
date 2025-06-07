use cpal::Sample;

use crate::SystemAudioInputError;

pub trait SystemAudioInput<S: Sample> {
    fn read_sample(&mut self) -> Result<S, SystemAudioInputError>;

    #[cfg(feature = "mock")]
    fn producer(&mut self) -> Option<Producer<S>>;
}
