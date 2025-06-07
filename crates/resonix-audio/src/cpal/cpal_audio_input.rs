use alloc::boxed::Box;
use cpal::Sample;

use crate::{Consumer, CpalAudioInputError, Producer, SystemAudioInput, SystemAudioInputError};

pub struct CpalAudioInput<S: Sample> {
    consumer: Consumer<S>,
    // TODO: will use to receive data from cpal
    #[allow(dead_code)]
    producer: Option<Producer<S>>,
}

impl<S: Sample> CpalAudioInput<S> {}

impl<S: Sample> SystemAudioInput<S> for CpalAudioInput<S> {
    fn read_sample(&mut self) -> Result<S, SystemAudioInputError> {
        let sample = self.consumer.read().map_err(|_| {
            SystemAudioInputError::ReadError(Box::new(CpalAudioInputError::ReadError))
        })?;

        Ok(sample)
    }

    #[cfg(feature = "mock")]
    fn producer(&mut self) -> Option<Producer<S>> {
        // only used in Mock implementation
        None
    }
}
