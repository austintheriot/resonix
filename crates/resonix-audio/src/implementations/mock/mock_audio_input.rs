use core::marker::PhantomData;

use alloc::{boxed::Box, vec::Vec};

use crate::{
    SystemAudioInputError,
    implementations::mock::MockAudioInputError,
    traits::{Consumer, Producer, SystemAudioInput},
};

pub struct MockAudioInput<S, C: Consumer<S>, P: Producer<S>> {
    consumer: C,
    producer: Option<P>,
    phantom: PhantomData<S>,
}

impl<S, C: Consumer<S> + Default, P: Producer<S> + Default> Default for MockAudioInput<S, C, P> {
    fn default() -> Self {
        Self {
            consumer: C::default(),
            producer: Some(P::default()),
            phantom: PhantomData,
        }
    }
}

impl<S, C: Consumer<S>, P: Producer<S>> MockAudioInput<S, C, P> {
    pub fn from_consumer_and_producer(consumer: C, producer: P) -> Self {
        Self {
            consumer,
            producer: Some(producer),
            phantom: PhantomData,
        }
    }
}

impl<S, C: Consumer<S>, P: Producer<S> + 'static> SystemAudioInput<S> for MockAudioInput<S, C, P> {
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

    fn producer(&mut self) -> Option<Box<dyn Producer<S>>> {
        self.producer
            .take()
            .map(|p| Box::new(p) as Box<dyn Producer<S>>)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_read_sample_from_empty_input_returns_error() {
        let mut input = MockAudioInput::<f32>::new();
        // producer is not used, so nothing was written
        assert!(input.try_read_sample().is_err());
    }

    #[test]
    fn producer_can_only_be_taken_once() {
        let mut input = MockAudioInput::<f32>::new();
        let first_producer = input.producer();
        let second_producer = input.producer();
        assert!(first_producer.is_some());
        assert!(second_producer.is_none());
    }

    #[test]
    fn producer_write_and_try_read_sample_roundtrip() {
        let mut input = MockAudioInput::<f32>::new();
        input.producer().unwrap().try_write(0.25f32).unwrap();
        let sample = input.try_read_sample().unwrap();
        assert_eq!(sample, 0.25f32);
    }

    #[test]
    fn read_into_fills_destination_slice_with_available_samples() {
        let mut input = MockAudioInput::<f32>::new();
        let mut producer = input.producer().unwrap();
        producer.try_write(1.0f32).unwrap();
        producer.try_write(2.0f32).unwrap();
        drop(producer);

        let mut destination = [0.0f32; 4];
        let count = input.read_into(&mut destination).unwrap();
        assert_eq!(count, 2);
        assert_eq!(destination[0], 1.0f32);
        assert_eq!(destination[1], 2.0f32);
    }

    #[test]
    fn read_into_returns_zero_when_no_samples_are_available() {
        let mut input = MockAudioInput::<f32>::new();
        let mut destination = [0.0f32; 4];
        let count = input.read_into(&mut destination).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn drain_collects_all_available_samples() {
        let mut input = MockAudioInput::<f32>::new();
        let mut producer = input.producer().unwrap();
        producer.try_write(5.0f32).unwrap();
        producer.try_write(10.0f32).unwrap();
        drop(producer);

        let samples = input.drain().unwrap();
        assert_eq!(samples, alloc::vec![5.0f32, 10.0f32]);
    }

    #[test]
    fn drain_on_empty_input_returns_empty_vec() {
        let mut input = MockAudioInput::<f32>::new();
        let samples = input.drain().unwrap();
        assert!(samples.is_empty());
    }
}
