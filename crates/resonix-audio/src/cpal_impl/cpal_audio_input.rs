use alloc::{boxed::Box, vec::Vec};
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
    fn try_read_sample(&mut self) -> Result<S, SystemAudioInputError> {
        let sample = self.consumer.try_read().map_err(|_| {
            SystemAudioInputError::ReadError(Box::new(CpalAudioInputError::ReadError))
        })?;

        Ok(sample)
    }

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

        while let Ok(sample) = self.try_read_sample() {
            out.push(sample);
        }

        Ok(out)
    }

    #[cfg(feature = "mock")]
    fn producer(&mut self) -> Option<Producer<S>> {
        // only used in Mock implementation
        None
    }
}
