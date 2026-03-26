use alloc::boxed::Box;

use crate::traits::{Consumer, Producer};

pub trait ChannelRuntime: 'static {
    fn create_channel<S: Send + 'static>(
        capacity: usize,
    ) -> (Box<dyn Producer<S> + Send>, Box<dyn Consumer<S> + Send>);
}
