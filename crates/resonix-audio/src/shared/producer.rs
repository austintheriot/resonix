use core::ops::{Deref, DerefMut};

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
