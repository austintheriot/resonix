use alloc::boxed::Box;

use crate::{
    SystemAudioOutputError,
    traits::{ChannelRuntime, Consumer, Producer, SystemAudioOutput},
};

pub struct MockAudioOutput<S> {
    producer: Box<dyn Producer<S> + Send>,
    consumer: Option<Box<dyn Consumer<S> + Send>>,
}

impl<S: Send + 'static> MockAudioOutput<S> {
    pub fn new<R: ChannelRuntime>() -> Self {
        let default_capacity = 2048;
        Self::with_capacity::<R>(default_capacity)
    }

    pub fn with_capacity<R: ChannelRuntime>(capacity: usize) -> Self {
        let (producer, consumer) = R::create_channel(capacity);
        Self {
            producer,
            consumer: Some(consumer),
        }
    }
}

impl<S: Copy> SystemAudioOutput<S> for MockAudioOutput<S> {
    fn try_write_sample(&mut self, sample: S) -> Result<(), SystemAudioOutputError> {
        self.producer.try_write(sample)?;

        Ok(())
    }

    fn consumer(&mut self) -> Option<Box<dyn Consumer<S> + Send>> {
        self.consumer.take()
    }

    fn ready_for_sample(&self) -> bool {
        self.producer.ready()
    }
}

#[cfg(test)]
mod tests {
    use crate::implementations::ringbuf::RingbufChannel;

    use super::*;

    #[test]
    fn ready_for_sample_returns_true_on_new_output() {
        let output = MockAudioOutput::<f32>::new::<RingbufChannel>();
        assert!(output.ready_for_sample());
    }

    #[test]
    fn try_write_sample_succeeds_when_buffer_has_space() {
        let mut output = MockAudioOutput::<f32>::new::<RingbufChannel>();
        assert!(output.try_write_sample(0.5f32).is_ok());
    }

    #[test]
    fn consumer_can_only_be_taken_once() {
        let mut output = MockAudioOutput::<f32>::new::<RingbufChannel>();
        let first_consumer = output.consumer();
        let second_consumer = output.consumer();
        assert!(first_consumer.is_some());
        assert!(second_consumer.is_none());
    }

    #[test]
    fn try_write_block_and_consumer_drain_roundtrip() {
        let mut output = MockAudioOutput::<f32>::new::<RingbufChannel>();
        let written_samples = [1.0f32, 2.0f32, 3.0f32];
        output.try_write_block(&written_samples).unwrap();

        let mut consumer = output.consumer().unwrap();
        let drained = consumer.drain();
        assert_eq!(drained, written_samples.as_slice());
    }

    #[test]
    fn try_write_sample_error_when_buffer_is_full() {
        let capacity = 4;
        let mut output = MockAudioOutput::<f32>::with_capacity::<RingbufChannel>(capacity);

        for _ in 0..capacity {
            output.try_write_sample(0.0f32).unwrap();
        }

        let result = output.try_write_sample(1.0f32);
        assert!(result.is_err());
    }

    #[test]
    fn ready_for_sample_returns_false_when_buffer_is_full() {
        let capacity = 4;
        let mut output = MockAudioOutput::<f32>::with_capacity::<RingbufChannel>(capacity);

        for _ in 0..capacity {
            output.try_write_sample(0.0f32).unwrap();
        }
        assert!(!output.ready_for_sample());
    }
}
