use alloc::boxed::Box;
use cpal::Sample;
use ringbuf::{
    HeapRb, SharedRb,
    storage::Heap,
    traits::{Producer, Split},
};

use crate::{MockAudioOutputError, SystemAudioOutput, SystemAudioOutputError};

pub struct MockAudioOutput<S: Sample> {
    producer: <SharedRb<Heap<S>> as Split>::Prod,
    consumer: Option<<SharedRb<Heap<S>> as Split>::Cons>,
}

impl<S: Sample> MockAudioOutput<S> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<S: Sample> Default for MockAudioOutput<S> {
    fn default() -> Self {
        let buffer = HeapRb::new(1024);
        let (producer, consumer) = buffer.split();
        Self {
            producer,
            consumer: Some(consumer),
        }
    }
}

impl<S: Sample> SystemAudioOutput<S> for MockAudioOutput<S> {
    fn write_sample(&mut self, sample: S) -> Result<(), SystemAudioOutputError> {
        self.producer.try_push(sample).map_err(|_sample| {
            SystemAudioOutputError::WriteError(Box::new(MockAudioOutputError::WriteError))
        })?;

        Ok(())
    }

    fn consumer(&mut self) -> Option<<SharedRb<Heap<S>> as Split>::Cons> {
        self.consumer.take()
    }
}
