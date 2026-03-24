pub struct Producer<S: Sample>(pub(crate) <SharedRb<Heap<S>> as Split>::Prod);

impl<S: Sample> Producer<S> {
    pub fn try_write(&mut self, sample: S) -> Result<(), ProducerError> {
        self.0
            .try_push(sample)
            .map_err(|_| ProducerError::WriteFailure)
    }

    pub fn try_write_block(&mut self, samples: &[S]) -> Result<(), ProducerError> {
        for &sample in samples.iter() {
            self.try_write(sample)?;
        }

        Ok(())
    }
}

impl<S: Sample> Deref for Producer<S> {
    type Target = <SharedRb<Heap<S>> as Split>::Prod;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S: Sample> DerefMut for Producer<S> {
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
    ) -> (Producer<f32>, impl RingBufConsumer<Item = f32>) {
        let ring_buffer = HeapRb::<f32>::new(capacity);
        let (producer, consumer) = ring_buffer.split();
        (Producer(producer), consumer)
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
        assert!(matches!(result, Err(ProducerError::WriteFailure)));
    }

    #[test]
    fn try_write_block_writes_all_samples_in_order() {
        let (mut producer, mut consumer) = make_producer_with_capacity(8);
        let samples = [1.0f32, 2.0f32, 3.0f32];
        producer.try_write_block(&samples).unwrap();
        assert_eq!(consumer.try_pop().unwrap(), 1.0f32);
        assert_eq!(consumer.try_pop().unwrap(), 2.0f32);
        assert_eq!(consumer.try_pop().unwrap(), 3.0f32);
    }

    #[test]
    fn try_write_block_fails_when_buffer_would_overflow() {
        let (mut producer, _consumer) = make_producer_with_capacity(2);
        let samples = [1.0f32, 2.0f32, 3.0f32]; // 3 samples, capacity 2
        let result = producer.try_write_block(&samples);
        assert!(result.is_err());
    }

    #[test]
    fn producer_error_display_message_is_correct() {
        let error = ProducerError::WriteFailure;
        assert_eq!(error.to_string(), "Failed to write sample");
    }
}
