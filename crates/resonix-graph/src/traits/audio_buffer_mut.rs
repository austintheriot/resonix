use crate::primitives::{AudioBufferError, Channel, Sample};

pub trait AudioBufferMut: crate::traits::AudioBuffer {
    fn channels_iter_mut(
        &mut self,
    ) -> Result<impl Iterator<Item = &mut [Sample]>, AudioBufferError>;

    fn mono_mut(&mut self) -> Result<&mut [Sample], AudioBufferError>;

    /// Returns the mutable samples for channel `c` (0-indexed).
    ///
    /// Should return `Err(ChannelOutOfRange)` if `c >= self.channels()`.
    fn channel_mut(
        &mut self,
        channel: impl Into<Channel>,
    ) -> Result<&mut [Sample], AudioBufferError>;
}
