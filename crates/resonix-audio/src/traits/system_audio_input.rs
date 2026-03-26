use alloc::vec::Vec;

use crate::SystemAudioInputError;

#[cfg(feature = "mock")]
use crate::traits::Producer;
#[cfg(feature = "mock")]
use alloc::boxed::Box;

pub trait SystemAudioInput<S> {
    /// Reads a single sample, if any is available
    fn try_read_sample(&mut self) -> Result<S, SystemAudioInputError>;

    /// Reads all available samples into a provided buffer
    fn read_into(&mut self, buffer: &mut [S]) -> Result<usize, SystemAudioInputError> {
        let mut count = 0;

        for slot in buffer.iter_mut() {
            match self.try_read_sample() {
                Ok(sample) => {
                    *slot = sample;
                    count += 1;
                }
                _ => break,
            }
        }

        Ok(count)
    }

    fn drain(&mut self) -> Result<Vec<S>, SystemAudioInputError> {
        let mut out = Vec::new();

        self.read_into(&mut out)?;

        Ok(out)
    }

    #[cfg(feature = "mock")]
    fn producer(&mut self) -> Option<Box<dyn Producer<S> + Send>>;
}
