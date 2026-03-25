use core::ops::{Deref, DerefMut};

use ringbuf::{
    SharedRb,
    storage::Heap,
    traits::{Observer, Producer as _, Split},
};

use crate::{ProducerError, traits::Producer};

pub struct RingbufProducer<S>(pub(crate) <SharedRb<Heap<S>> as Split>::Prod);

impl<S> RingbufProducer<S> {
    pub fn new(producer: <SharedRb<Heap<S>> as Split>::Prod) -> Self {
        Self(producer)
    }
}

impl<S> Producer<S> for RingbufProducer<S> {
    fn try_write(&mut self, sample: S) -> Result<(), ProducerError> {
        self.0
            .try_push(sample)
            .map_err(|_sample| ProducerError::WriteFailure(None))
    }

    fn ready(&self) -> bool {
        !self.0.is_full()
    }
}

impl<S> Deref for RingbufProducer<S> {
    type Target = <SharedRb<Heap<S>> as Split>::Prod;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> DerefMut for RingbufProducer<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use ringbuf::{HeapRb, traits::Consumer as RingBufConsumer};

    /// Creates a (Producer<f32>, reader) pair backed by a ringbuffer of the given capacity.
    fn make_producer_with_capacity(
        capacity: usize,
    ) -> (RingbufProducer<f32>, impl RingBufConsumer<Item = f32>) {
        let ring_buffer = HeapRb::<f32>::new(capacity);
        let (producer, consumer) = ring_buffer.split();
        (RingbufProducer(producer), consumer)
    }

    #[test]
    fn try_write_pushes_a_single_sample_to_the_buffer() {
        let (mut producer, mut consumer) = make_producer_with_capacity(4);
        producer.try_write(0.75f32).unwrap();
        assert_eq!(consumer.try_pop().unwrap(), 0.75f32);
    }

    #[test]
    fn try_write_to_full_buffer_returns_write_failure_error() {
        let (mut producer, _consumer) = make_producer_with_capacity(1);
        producer.try_write(1.0f32).unwrap();
        let result = producer.try_write(2.0f32);
        assert!(matches!(result, Err(ProducerError::WriteFailure(..))));
    }

    #[test]
    fn producer_error_display_message_is_correct() {
        let error = ProducerError::WriteFailure(None);
        assert_eq!(error.to_string(), "Failed to write sample");
    }
}
