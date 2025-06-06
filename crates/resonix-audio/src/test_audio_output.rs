use alloc::boxed::Box;
use cpal::Sample;
use ringbuf::{
    HeapRb, SharedRb,
    storage::Heap,
    traits::{Producer, Split},
};

use crate::{SystemAudioOutput, SystemAudioOutputError, TestAudioOutputError};

pub struct TestAudioOutput<S: Sample> {
    producer: <SharedRb<Heap<S>> as Split>::Prod,
}

impl<S: Sample> TestAudioOutput<S> {
    pub fn new() -> (Self, <SharedRb<Heap<S>> as Split>::Cons) {
        let buffer = HeapRb::new(1024);
        let (producer, consumer) = buffer.split();
        (Self { producer }, consumer)
    }
}

impl<S: Sample> SystemAudioOutput<S> for TestAudioOutput<S> {
    fn write_sample(&mut self, sample: S) -> Result<(), SystemAudioOutputError> {
        self.producer.try_push(sample).map_err(|_sample| {
            SystemAudioOutputError::WriteError(Box::new(TestAudioOutputError::WriteError))
        })?;

        Ok(())
    }
}
