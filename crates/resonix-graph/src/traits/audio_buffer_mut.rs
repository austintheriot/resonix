use crate::errors::AudioBufferError;
use crate::primitives::{Channel, Sample};

pub trait AudioBufferMut: crate::traits::AudioBuffer {
    /// Returns the mutable samples slice for `channel` (0-indexed).
    ///
    /// Should return `Err(ChannelOutOfRange)` if `c >= self.channels()`.
    fn channel_mut(
        &mut self,
        channel: impl Into<Channel>,
    ) -> Result<&mut [Sample], AudioBufferError>;

    /// Returns an iterator over each channel in the audio buffer.
    ///
    /// Iterates from the 1st channel (channel `0`) to the highest.
    fn channels_iter_mut(
        &mut self,
    ) -> Result<impl Iterator<Item = &mut [Sample]>, AudioBufferError>;

    /// Returns the single channel's samples.
    ///
    /// Should return `Err(NotMono)` if `channels != 1`.
    fn mono_mut(&mut self) -> Result<&mut [Sample], AudioBufferError>;

    /// Returns the internal audio buffer as a contiguous, mutable slice,
    /// without checking the number of channels it contains.
    fn as_slice_mut(&mut self) -> &mut [Sample];
}
