use cpal::Sample;

use ringbuf::{
    SharedRb,
    storage::Heap,
    traits::{Consumer as RingBufConsumer, Split},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConsumerError {
    #[error("Failed to read sample")]
    ReadFailure,
}

pub struct Consumer<S: Sample>(pub(crate) <SharedRb<Heap<S>> as Split>::Cons);

impl<S: Sample> Consumer<S> {
    pub fn read(&mut self) -> Result<S, ConsumerError> {
        self.0.try_pop().ok_or(ConsumerError::ReadFailure)
    }
}
