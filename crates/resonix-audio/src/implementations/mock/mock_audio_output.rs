use core::marker::PhantomData;

use alloc::boxed::Box;

use crate::{
    SystemAudioOutputError,
    implementations::mock::MockAudioOutputError,
    traits::{Consumer, Producer, SystemAudioOutput},
};

pub struct MockAudioOutput<S, P: Producer<S>, C: Consumer<S>> {
    producer: P,
    consumer: Option<C>,
    phantom: PhantomData<S>,
}

impl<S, C: Consumer<S>, P: Producer<S>> MockAudioOutput<S, P, C> {
    pub fn from_producer_and_consumer(producer: P, consumer: C) -> Self {
        Self {
            producer,
            consumer: Some(consumer),
            phantom: PhantomData,
        }
    }
}

impl<S, C: Consumer<S> + Default, P: Producer<S> + Default> Default for MockAudioOutput<S, P, C> {
    fn default() -> Self {
        Self {
            producer: P::default(),
            consumer: Some(C::default()),
            phantom: PhantomData,
        }
    }
}

impl<S, C: Consumer<S>, P: Producer<S>> SystemAudioOutput<S> for MockAudioOutput<S, P, C> {
    fn try_write_block(&mut self, samples: &[S]) -> Result<(), SystemAudioOutputError> {
        self.producer.try_write_block(samples).map_err(|_e| {
            SystemAudioOutputError::WriteError(Box::new(MockAudioOutputError::WriteError))
        })?;

        Ok(())
    }

    fn try_write_sample(&mut self, sample: S) -> Result<(), SystemAudioOutputError> {
        self.producer.try_push(sample).map_err(|_sample| {
            SystemAudioOutputError::WriteError(Box::new(MockAudioOutputError::WriteError))
        })?;

        Ok(())
    }

    fn consumer(&mut self) -> Option<Box<dyn Consumer<S>>> {
        self.consumer
            .take()
            .map(|c| Box::new(c) as Box<dyn Consumer<S>>)
    }

    fn ready_for_sample(&self) -> bool {
        !self.producer.is_full()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ready_for_sample_returns_true_on_new_output() {
        let output = MockAudioOutput::<f32>::new();
        assert!(output.ready_for_sample());
    }

    #[test]
    fn try_write_sample_succeeds_when_buffer_has_space() {
        let mut output = MockAudioOutput::<f32>::new();
        assert!(output.try_write_sample(0.5f32).is_ok());
    }

    #[test]
    fn consumer_can_only_be_taken_once() {
        let mut output = MockAudioOutput::<f32>::new();
        let first_consumer = output.consumer();
        let second_consumer = output.consumer();
        assert!(first_consumer.is_some());
        assert!(second_consumer.is_none());
    }

    #[test]
    fn try_write_block_and_consumer_drain_roundtrip() {
        let mut output = MockAudioOutput::<f32>::new();
        let written_samples = [1.0f32, 2.0f32, 3.0f32];
        output.try_write_block(&written_samples).unwrap();

        let mut consumer = output.consumer().unwrap();
        let drained = consumer.drain();
        assert_eq!(drained, written_samples.as_slice());
    }

    #[test]
    fn try_write_sample_error_when_buffer_is_full() {
        let mut output = MockAudioOutput::<f32>::new();
        // Fill the entire 1024-sample capacity
        for _ in 0..1024 {
            output.try_write_sample(0.0f32).unwrap();
        }
        let result = output.try_write_sample(1.0f32);
        assert!(result.is_err());
    }

    #[test]
    fn ready_for_sample_returns_false_when_buffer_is_full() {
        let mut output = MockAudioOutput::<f32>::new();
        for _ in 0..1024 {
            output.try_write_sample(0.0f32).unwrap();
        }
        assert!(!output.ready_for_sample());
    }
}
