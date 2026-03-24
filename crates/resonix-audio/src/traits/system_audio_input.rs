use alloc::vec::Vec;

use crate::SystemAudioInputError;

#[cfg(feature = "mock")]
use crate::traits::Producer;
#[cfg(feature = "mock")]
use alloc::boxed::Box;

pub trait SystemAudioInput<S> {
    fn drain(&mut self) -> Result<Vec<S>, SystemAudioInputError>;

    /// Reads all available samples into a provided buffer
    fn read_into(&mut self, buffer: &mut [S]) -> Result<usize, SystemAudioInputError>;

    /// Reads a single sample, if any is available
    fn try_read_sample(&mut self) -> Result<S, SystemAudioInputError>;

    #[cfg(feature = "mock")]
    fn producer(&mut self) -> Option<Box<dyn Producer<S>>>;
}
