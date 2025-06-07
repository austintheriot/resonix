use cpal::Sample;

#[cfg(feature = "mock")]
use ringbuf::{SharedRb, storage::Heap, traits::Split};

use crate::SystemAudioOutputError;

pub trait SystemAudioOutput<S: Sample> {
    fn write_sample(&mut self, sample: S) -> Result<(), SystemAudioOutputError>;

    #[cfg(feature = "mock")]
    fn consumer(&mut self) -> Option<<SharedRb<Heap<S>> as Split>::Cons>;
}
