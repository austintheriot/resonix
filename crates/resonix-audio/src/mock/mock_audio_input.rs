use alloc::boxed::Box;
use cpal::Sample;
use ringbuf::{
    HeapRb, SharedRb,
    storage::Heap,
    traits::{Consumer, Split},
};

use crate::{MockAudioInputError, SystemAudioInput, SystemAudioInputError};

pub struct MockAudioInput<S: Sample> {
    consumer: <SharedRb<Heap<S>> as Split>::Cons,
    producer: Option<<SharedRb<Heap<S>> as Split>::Prod>,
}

impl<S: Sample> Default for MockAudioInput<S> {
    fn default() -> Self {
        let buffer = HeapRb::new(1024);
        let (producer, consumer) = buffer.split();
        Self {
            consumer,
            producer: Some(producer),
        }
    }
}

impl<S: Sample> MockAudioInput<S> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<S: Sample> SystemAudioInput<S> for MockAudioInput<S> {
    fn read_sample(&mut self) -> Result<S, SystemAudioInputError> {
        let sample = self.consumer.try_pop().ok_or_else(|| {
            SystemAudioInputError::ReadError(Box::new(MockAudioInputError::ReadError))
        })?;

        Ok(sample)
    }

    fn producer(&mut self) -> Option<<SharedRb<Heap<S>> as Split>::Prod> {
        self.producer.take()
    }
}
