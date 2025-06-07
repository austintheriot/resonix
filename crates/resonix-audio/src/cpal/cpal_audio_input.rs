use alloc::boxed::Box;
use cpal::Sample;
use ringbuf::{
    HeapRb, SharedRb,
    storage::Heap,
    traits::{Consumer, Split},
};

use crate::{CpalAudioInputError, SystemAudioInput, SystemAudioInputError};

pub struct CpalAudioInput<S: Sample> {
    consumer: <SharedRb<Heap<S>> as Split>::Cons,
}

impl<S: Sample> CpalAudioInput<S> {
    pub fn new() -> (Self, <SharedRb<Heap<S>> as Split>::Prod) {
        let buffer = HeapRb::new(1024);
        let (producer, consumer) = buffer.split();
        (Self { consumer }, producer)
    }
}

impl<S: Sample> SystemAudioInput<S> for CpalAudioInput<S> {
    fn read_sample(&mut self) -> Result<S, SystemAudioInputError> {
        let sample = self.consumer.try_pop().ok_or_else(|| {
            SystemAudioInputError::ReadError(Box::new(CpalAudioInputError::ReadError))
        })?;

        Ok(sample)
    }

    #[cfg(feature = "mock")]
    fn producer(&mut self) -> Option<<SharedRb<Heap<S>> as Split>::Prod> {
        // only used in Mock implementation
        None
    }
}
