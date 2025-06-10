use alloc::boxed::Box;
use cpal::Sample;
use ringbuf::{
    HeapRb,
    traits::{Observer, Producer as RingBufProducer, Split},
};

use crate::{Consumer, MockAudioOutputError, Producer, SystemAudioOutput, SystemAudioOutputError};

pub struct MockAudioOutput<S: Sample> {
    producer: Producer<S>,
    consumer: Option<Consumer<S>>,
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
            producer: Producer(producer),
            consumer: Some(Consumer(consumer)),
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

    fn consumer(&mut self) -> Option<Consumer<S>> {
        self.consumer.take()
    }

    fn ready_for_sample(&self) -> bool {
        !self.producer.is_full()
    }
}
