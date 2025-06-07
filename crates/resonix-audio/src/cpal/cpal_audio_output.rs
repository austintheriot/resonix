use alloc::boxed::Box;
use cpal::Sample;

use crate::{Consumer, CpalAudioOutputError, Producer, SystemAudioOutput, SystemAudioOutputError};

pub struct CpalAudioOutput<S: Sample> {
    producer: Producer<S>,
    // TODO: will use to send data to cpal
    #[allow(dead_code)]
    consumer: Option<Consumer<S>>,
}

impl<S: Sample> CpalAudioOutput<S> {}

impl<S> SystemAudioOutput<S> for CpalAudioOutput<S>
where
    S: Sample,
{
    fn write_sample(&mut self, sample: S) -> Result<(), SystemAudioOutputError> {
        self.producer.write(sample).map_err(|_sample| {
            SystemAudioOutputError::WriteError(Box::new(CpalAudioOutputError::WriteError))
        })?;

        Ok(())
    }

    #[cfg(feature = "mock")]
    fn consumer(&mut self) -> Option<Consumer<S>> {
        // only used in Mock implementation
        None
    }
}
