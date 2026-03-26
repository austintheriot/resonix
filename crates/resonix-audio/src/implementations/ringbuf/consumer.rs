use ringbuf::{
    SharedRb,
    storage::Heap,
    traits::{Consumer as _, Split},
};

use crate::{ConsumerError, traits::Consumer};

/// Typed wrapper around the ringbuf split consumer
pub struct RingbufConsumer<S>(pub(crate) <SharedRb<Heap<S>> as Split>::Cons);

impl<S> RingbufConsumer<S> {
    pub fn new(value: <SharedRb<Heap<S>> as Split>::Cons) -> Self {
        Self(value)
    }
}

impl<S> Consumer<S> for RingbufConsumer<S> {
    fn try_read(&mut self) -> Result<S, ConsumerError> {
        self.0.try_pop().ok_or(ConsumerError::NoData)
    }
}

#[cfg(test)]
mod tests {
    use crate::implementations::ringbuf::RingbufChannel;

    use super::*;
    use ringbuf::traits::Producer as _;

    #[test]
    fn try_read_from_empty_buffer_returns_read_failure_error() {
        let (_producer, mut consumer) = RingbufChannel::create_named_channel::<f32>(4);

        assert!(matches!(consumer.try_read(), Err(ConsumerError::NoData)));
    }

    #[test]
    fn try_read_after_write_returns_the_written_sample() {
        let (mut producer, mut consumer) = RingbufChannel::create_named_channel::<f32>(4);

        producer.try_push(0.5f32).unwrap();

        assert_eq!(consumer.try_read().unwrap(), 0.5f32);
    }

    #[test]
    fn try_read_returns_samples_in_fifo_order() {
        let (mut producer, mut consumer) = RingbufChannel::create_named_channel::<f32>(4);

        producer.try_push(1.0f32).unwrap();
        producer.try_push(2.0f32).unwrap();

        assert_eq!(consumer.try_read().unwrap(), 1.0f32);
        assert_eq!(consumer.try_read().unwrap(), 2.0f32);
    }

    #[test]
    fn read_into_empty_buffer_writes_zero_samples_and_returns_zero() {
        let (_producer, mut consumer) = RingbufChannel::create_named_channel::<f32>(4);

        let mut destination = [0.0f32; 4];
        let count = consumer.read_into(&mut destination);

        assert_eq!(count, 0);
    }

    #[test]
    fn read_into_fills_at_most_the_available_samples() {
        const CAPACITY: usize = 8;
        let (mut producer, mut consumer) = RingbufChannel::create_named_channel::<f32>(CAPACITY);

        producer.try_push(1.0f32).unwrap();
        producer.try_push(2.0f32).unwrap();
        let mut destination = [0.0f32; CAPACITY];
        let count = consumer.read_into(&mut destination);

        assert_eq!(count, 2);
        assert_eq!(destination[0], 1.0f32);
        assert_eq!(destination[1], 2.0f32);
    }

    #[test]
    fn read_into_does_not_write_beyond_the_destination_slice_length() {
        const CAPACITY: usize = 4;
        let (mut producer, mut consumer) = RingbufChannel::create_named_channel::<f32>(CAPACITY);

        producer.try_push(1.0f32).unwrap();
        producer.try_push(2.0f32).unwrap();
        producer.try_push(3.0f32).unwrap();
        let mut destination = [0.0f32; 2];
        let count = consumer.read_into(&mut destination);

        assert_eq!(count, 2);
    }

    #[test]
    fn drain_on_empty_buffer_returns_empty_vec() {
        const CAPACITY: usize = 4;
        let (_producer, mut consumer) = RingbufChannel::create_named_channel::<f32>(CAPACITY);

        let drained = consumer.drain();
        assert!(drained.is_empty());
    }

    #[test]
    fn drain_collects_all_available_samples() {
        const CAPACITY: usize = 8;
        let (mut producer, mut consumer) = RingbufChannel::create_named_channel::<f32>(CAPACITY);

        producer.try_push(10.0f32).unwrap();
        producer.try_push(20.0f32).unwrap();
        producer.try_push(30.0f32).unwrap();
        let drained = consumer.drain();

        assert_eq!(drained, alloc::vec![10.0f32, 20.0f32, 30.0f32]);
    }

    #[test]
    fn drain_leaves_buffer_empty_afterwards() {
        const CAPACITY: usize = 8;
        let (mut producer, mut consumer) = RingbufChannel::create_named_channel::<f32>(CAPACITY);

        producer.try_push(1.0f32).unwrap();
        let _ = consumer.drain();
        let second_drain = consumer.drain();

        assert!(second_drain.is_empty());
    }
}
