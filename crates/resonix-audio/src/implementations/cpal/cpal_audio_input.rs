use core::marker::PhantomData;

use alloc::{boxed::Box, vec::Vec};
use cpal::Sample;

use crate::{
    SystemAudioInputError,
    traits::{Consumer, SystemAudioInput},
};

#[cfg(feature = "mock")]
use crate::traits::Producer;

pub struct CpalAudioInput<S: Sample + Send + 'static> {
    consumer: Box<dyn Consumer<S>>,
    _phantom: PhantomData<S>,
}

impl<S: Sample + Send + 'static> SystemAudioInput<S> for CpalAudioInput<S> {
    fn try_read_sample(&mut self) -> Result<S, SystemAudioInputError> {
        let sample = self.consumer.try_read()?;

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
    fn producer(&mut self) -> Option<Box<dyn Producer<S> + Send>> {
        // only used in Mock implementation--we need the producer
        // to be able to get audio data from cpal
        None
    }
}
