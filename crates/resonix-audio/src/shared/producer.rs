use cpal::Sample;

use ringbuf::{
    SharedRb,
    storage::Heap,
    traits::{Producer as RingBufProducer, Split},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProducerError {
    #[error("Failed to write sample")]
    WriteFailure,
}

pub struct Producer<S: Sample>(pub(crate) <SharedRb<Heap<S>> as Split>::Prod);

impl<S: Sample> Producer<S> {
    pub fn write(&mut self, sample: S) -> Result<(), ProducerError> {
        self.0
            .try_push(sample)
            .map_err(|_| ProducerError::WriteFailure)
    }
}
