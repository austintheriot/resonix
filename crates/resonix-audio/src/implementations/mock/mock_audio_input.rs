use core::marker::PhantomData;

use alloc::boxed::Box;

use crate::{
    SystemAudioInputError,
    traits::{ChannelRuntime, Consumer, Producer, SystemAudioInput},
};

pub struct MockAudioInput<S> {
    consumer: Box<dyn Consumer<S> + Send>,
    producer: Option<Box<dyn Producer<S> + Send>>,
    phantom: PhantomData<S>,
}

impl<S: Send + 'static> MockAudioInput<S> {
    pub fn new<R: ChannelRuntime>() -> Self {
        // TODO: share this somewhere? pretty arbitrary at this point
        let default_capacity = 2048;
        Self::with_capacity::<R>(default_capacity)
    }

    pub fn with_capacity<R: ChannelRuntime>(capacity: usize) -> Self {
        let (producer, consumer) = R::create_channel(capacity);
        Self {
            consumer,
            producer: Some(producer),
            phantom: PhantomData,
        }
    }
}

impl<S: Send + 'static> SystemAudioInput<S> for MockAudioInput<S> {
    fn try_read_sample(&mut self) -> Result<S, SystemAudioInputError> {
        let sample = self.consumer.try_read()?;

        Ok(sample)
    }

    fn producer(&mut self) -> Option<Box<dyn Producer<S> + Send>> {
        self.producer.take()
    }
}

#[cfg(test)]
mod tests {
    use crate::implementations::ringbuf::RingbufChannel;

    use super::*;

    #[test]
    fn try_read_sample_from_empty_input_returns_error() {
        let mut input = MockAudioInput::<f32>::new::<RingbufChannel>();
        // producer is not used, so nothing was written
        assert!(input.try_read_sample().is_err());
    }

    #[test]
    fn producer_can_only_be_taken_once() {
        let mut input = MockAudioInput::<f32>::new::<RingbufChannel>();
        let first_producer = input.producer();
        let second_producer = input.producer();
        assert!(first_producer.is_some());
        assert!(second_producer.is_none());
    }

    #[test]
    fn producer_write_and_try_read_sample_roundtrip() {
        let mut input = MockAudioInput::<f32>::new::<RingbufChannel>();
        input.producer().unwrap().try_write(0.25f32).unwrap();
        let sample = input.try_read_sample().unwrap();
        assert_eq!(sample, 0.25f32);
    }

    #[test]
    fn read_into_fills_destination_slice_with_available_samples() {
        let mut input = MockAudioInput::<f32>::new::<RingbufChannel>();
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
        let mut input = MockAudioInput::<f32>::new::<RingbufChannel>();
        let mut destination = [0.0f32; 4];
        let count = input.read_into(&mut destination).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn drain_collects_all_available_samples() {
        let mut input = MockAudioInput::<f32>::new::<RingbufChannel>();
        let mut producer = input.producer().unwrap();
        producer.try_write(5.0f32).unwrap();
        producer.try_write(10.0f32).unwrap();
        drop(producer);

        let samples = input.drain().unwrap();
        assert_eq!(samples, alloc::vec![5.0f32, 10.0f32]);
    }

    #[test]
    fn drain_on_empty_input_returns_empty_vec() {
        let mut input = MockAudioInput::<f32>::new::<RingbufChannel>();
        let samples = input.drain().unwrap();
        assert!(samples.is_empty());
    }
}
