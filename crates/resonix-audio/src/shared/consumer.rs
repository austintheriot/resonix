use alloc::vec::Vec;
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
    pub fn try_read(&mut self) -> Result<S, ConsumerError> {
        self.0.try_pop().ok_or(ConsumerError::ReadFailure)
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
