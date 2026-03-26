use crate::{
    implementations::ringbuf::{RingbufConsumer, RingbufProducer},
    traits::{ChannelRuntime, Consumer, Producer},
};

use alloc::boxed::Box;
use ringbuf::{HeapRb, traits::Split};

pub struct RingbufRuntime;

impl RingbufRuntime {
    /// Raw type without type erasure: useful for internal tests
    pub fn create_named_channel<S: Send + 'static>(
        capacity: usize,
    ) -> (RingbufProducer<S>, RingbufConsumer<S>) {
        let channels = HeapRb::new(capacity);
        let (producer, consumer) = channels.split();
        (RingbufProducer(producer), RingbufConsumer(consumer))
    }
}

impl ChannelRuntime for RingbufRuntime {
    fn create_channel<S: Send + 'static>(
        capacity: usize,
    ) -> (Box<dyn Producer<S> + Send>, Box<dyn Consumer<S> + Send>) {
        let (producer, consumer) = Self::create_named_channel(capacity);
        (
            Box::new(producer) as Box<dyn Producer<S> + Send>,
            Box::new(consumer) as Box<dyn Consumer<S> + Send>,
        )
    }
}
