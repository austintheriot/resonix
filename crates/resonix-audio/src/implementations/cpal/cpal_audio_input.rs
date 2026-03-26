use core::marker::PhantomData;

use alloc::boxed::Box;
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

    #[cfg(feature = "mock")]
    fn producer(&mut self) -> Option<Box<dyn Producer<S> + Send>> {
        // only used in Mock implementation--we need the producer
        // to be able to get audio data from cpal
        None
    }
}
