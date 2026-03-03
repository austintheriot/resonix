use alloc::vec::Vec;
use cpal::Sample;

#[cfg(feature = "mock")]
use crate::Producer;

use crate::SystemAudioInputError;

pub trait SystemAudioInput<S: Sample> {
    fn drain(&mut self) -> Result<Vec<S>, SystemAudioInputError>;

    /// Reads all available samples into a provided buffer
    fn read_into(&mut self, buffer: &mut [S]) -> Result<usize, SystemAudioInputError>;

    /// Reads a single sample, if any is available
    fn try_read_sample(&mut self) -> Result<S, SystemAudioInputError>;

    #[cfg(feature = "mock")]
    fn producer(&mut self) -> Option<Producer<S>>;
}
