pub struct Consumer<S: Sample>(pub(crate) <SharedRb<Heap<S>> as Split>::Cons);

impl<S: Sample> Consumer<S> {
    pub fn try_read(&mut self) -> Result<S, ConsumerError> {
        self.0.try_pop().ok_or(ConsumerError::NoData)
    }

    /// Reads as many samples as possible into `buf`.
    /// Returns the number of samples written.
    pub fn read_into(&mut self, buffer: &mut [S]) -> usize {
        let mut count = 0;

        for slot in buffer.iter_mut() {
            match self.0.try_pop() {
                Some(sample) => {
                    *slot = sample;
                    count += 1;
                }
                None => break,
            }
        }

        count
    }

    /// Drains all currently available samples into a Vec.
    pub fn drain(&mut self) -> Vec<S> {
        let mut out = Vec::new();

        while let Some(sample) = self.0.try_pop() {
            out.push(sample);
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use ringbuf::{HeapRb, traits::Producer as RingBufProducer};

    /// Creates a (writer, Consumer<f32>) pair backed by a ringbuffer of the given capacity.
    fn make_consumer_with_capacity(
        capacity: usize,
    ) -> (impl RingBufProducer<Item = f32>, Consumer<f32>) {
        let ring_buffer = HeapRb::<f32>::new(capacity);
        let (producer, consumer) = ring_buffer.split();
        (producer, Consumer(consumer))
    }

    #[test]
    fn try_read_from_empty_buffer_returns_read_failure_error() {
        let (_producer, mut consumer) = make_consumer_with_capacity(4);
        assert!(matches!(
            consumer.try_read(),
            Err(ConsumerError::ReadFailure)
        ));
    }

    #[test]
    fn try_read_after_write_returns_the_written_sample() {
        let (mut producer, mut consumer) = make_consumer_with_capacity(4);
        producer.try_push(0.5f32).unwrap();
        assert_eq!(consumer.try_read().unwrap(), 0.5f32);
    }

    #[test]
    fn try_read_returns_samples_in_fifo_order() {
        let (mut producer, mut consumer) = make_consumer_with_capacity(4);
        producer.try_push(1.0f32).unwrap();
        producer.try_push(2.0f32).unwrap();
        assert_eq!(consumer.try_read().unwrap(), 1.0f32);
        assert_eq!(consumer.try_read().unwrap(), 2.0f32);
    }

    #[test]
    fn read_into_empty_buffer_writes_zero_samples_and_returns_zero() {
        let (_producer, mut consumer) = make_consumer_with_capacity(4);
        let mut destination = [0.0f32; 4];
        let count = consumer.read_into(&mut destination);
        assert_eq!(count, 0);
    }

    #[test]
    fn read_into_fills_at_most_the_available_samples() {
        let (mut producer, mut consumer) = make_consumer_with_capacity(8);
        producer.try_push(1.0f32).unwrap();
        producer.try_push(2.0f32).unwrap();
        let mut destination = [0.0f32; 4];
        let count = consumer.read_into(&mut destination);
        assert_eq!(count, 2);
        assert_eq!(destination[0], 1.0f32);
        assert_eq!(destination[1], 2.0f32);
    }

    #[test]
    fn read_into_does_not_write_beyond_the_destination_slice_length() {
        let (mut producer, mut consumer) = make_consumer_with_capacity(8);
        producer.try_push(1.0f32).unwrap();
        producer.try_push(2.0f32).unwrap();
        producer.try_push(3.0f32).unwrap();
        let mut destination = [0.0f32; 2];
        let count = consumer.read_into(&mut destination);
        assert_eq!(count, 2);
    }

    #[test]
    fn drain_on_empty_buffer_returns_empty_vec() {
        let (_producer, mut consumer) = make_consumer_with_capacity(4);
        let drained = consumer.drain();
        assert!(drained.is_empty());
    }

    #[test]
    fn drain_collects_all_available_samples() {
        let (mut producer, mut consumer) = make_consumer_with_capacity(8);
        producer.try_push(10.0f32).unwrap();
        producer.try_push(20.0f32).unwrap();
        producer.try_push(30.0f32).unwrap();
        let drained = consumer.drain();
        assert_eq!(drained, alloc::vec![10.0f32, 20.0f32, 30.0f32]);
    }

    #[test]
    fn drain_leaves_buffer_empty_afterwards() {
        let (mut producer, mut consumer) = make_consumer_with_capacity(4);
        producer.try_push(1.0f32).unwrap();
        let _ = consumer.drain();
        let second_drain = consumer.drain();
        assert!(second_drain.is_empty());
    }

    #[test]
    fn consumer_error_display_message_is_correct() {
        let error = ConsumerError::ReadFailure;
        assert_eq!(error.to_string(), "Failed to read sample");
    }
}
