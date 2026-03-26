use core::ops::{Deref, DerefMut};

use ringbuf::{
    SharedRb,
    storage::Heap,
    traits::{Observer, Producer as _, Split},
};

use crate::{ProducerError, traits::Producer};

/// Typed wrapper around the ringbuf split producer
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
            .map_err(|_sample| ProducerError::InsufficientSpace)
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
    use crate::{implementations::ringbuf::RingbufChannel, traits::Consumer};

    use super::*;

    #[test]
    fn try_write_pushes_a_single_sample_to_the_buffer() {
        let (mut producer, mut consumer) = RingbufChannel::create_named_channel(4);

        producer.try_write(0.75f32).unwrap();

        assert_eq!(consumer.try_read().unwrap(), 0.75f32);
    }

    #[test]
    fn try_write_to_full_buffer_returns_write_failure_error() {
        let (mut producer, _consumer) = RingbufChannel::create_named_channel(1);

        producer.try_write(1.0f32).unwrap();
        let result = producer.try_write(2.0f32);

        assert!(matches!(result, Err(ProducerError::InsufficientSpace)));
    }
}
