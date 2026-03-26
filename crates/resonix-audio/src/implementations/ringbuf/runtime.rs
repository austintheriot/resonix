use crate::{
    implementations::ringbuf::{RingbufConsumer, RingbufProducer},
    traits::{ChannelRuntime, Consumer, Producer},
};

use alloc::boxed::Box;
use ringbuf::{HeapRb, traits::Split};

pub struct RingbufRuntime;

impl ChannelRuntime for RingbufRuntime {
    fn create_channel<S: Send + 'static>(
        &self,
        capacity: usize,
    ) -> (Box<dyn Producer<S> + Send>, Box<dyn Consumer<S> + Send>) {
        let channels = HeapRb::new(capacity);
        let (producer, consumer) = channels.split();
        (
            Box::new(RingbufProducer(producer)) as Box<dyn Producer<S> + Send>,
            Box::new(RingbufConsumer(consumer)) as Box<dyn Consumer<S> + Send>,
        )
    }
}
