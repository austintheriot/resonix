use cpal::Sample;

#[cfg(feature = "mock")]
use ringbuf::{SharedRb, storage::Heap, traits::Split};

use crate::SystemAudioInputError;

pub trait SystemAudioInput<S: Sample> {
    fn read_sample(&mut self) -> Result<S, SystemAudioInputError>;

    #[cfg(feature = "mock")]
    fn producer(&mut self) -> Option<<SharedRb<Heap<S>> as Split>::Prod>;
}
