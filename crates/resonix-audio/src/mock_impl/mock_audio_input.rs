use alloc::{boxed::Box, vec::Vec};
use cpal::Sample;
use ringbuf::{
    HeapRb, SharedRb,
    storage::Heap,
    traits::{Consumer, Split},
};

use crate::{MockAudioInputError, Producer, SystemAudioInput, SystemAudioInputError};

pub struct MockAudioInput<S: Sample> {
    consumer: <SharedRb<Heap<S>> as Split>::Cons,
    producer: Option<Producer<S>>,
}

impl<S: Sample> Default for MockAudioInput<S> {
    fn default() -> Self {
        let buffer = HeapRb::new(1024);
        let (producer, consumer) = buffer.split();
        Self {
            consumer,
            producer: Some(Producer(producer)),
        }
    }
}

impl<S: Sample> MockAudioInput<S> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<S: Sample> SystemAudioInput<S> for MockAudioInput<S> {
    fn try_read_sample(&mut self) -> Result<S, SystemAudioInputError> {
        let sample = self.consumer.try_pop().ok_or_else(|| {
            // TODO: narrow down to out-of-data error
            SystemAudioInputError::ReadError(Box::new(MockAudioInputError::ReadError))
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

    fn producer(&mut self) -> Option<Producer<S>> {
        self.producer.take()
    }
}
