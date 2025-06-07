use alloc::boxed::Box;
use cpal::Sample;
use ringbuf::{HeapRb, traits::Producer};

use crate::{CpalAudioOutputError, SystemAudioOutput, SystemAudioOutputError};

pub struct CpalAudioOutput<S: Sample> {
    buffer: HeapRb<S>,
}

impl<S: Sample> CpalAudioOutput<S> {}

impl<S> SystemAudioOutput<S> for CpalAudioOutput<S>
where
    S: Sample,
{
    fn write_sample(&mut self, sample: S) -> Result<(), SystemAudioOutputError> {
        self.buffer.try_push(sample).map_err(|_sample| {
            SystemAudioOutputError::WriteError(Box::new(CpalAudioOutputError::WriteError))
        })?;

        Ok(())
    }
}
