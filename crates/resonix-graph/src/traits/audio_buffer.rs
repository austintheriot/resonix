use crate::errors::AudioBufferError;
use crate::primitives::{Channel, Sample};

pub trait AudioBuffer {
    /// Returns the length of each slice per channel.
    ///
    /// For example, if an `AudioBuffer` implementation has 2 channels
    /// and a block_size of `256`, then the total length of the
    /// `AudioBuffer` would be assumed to be `512`.
    fn block_size(&self) -> usize;

    /// Returns the number of channels in the audio buffer.
    fn channels(&self) -> usize;

    /// Returns the samples for `channel` (0-indexed).
    ///
    /// Should return `Err(ChannelOutOfRange)` if `c >= self.channels()`.
    fn channel(&self, channel: impl Into<Channel>) -> Result<&[Sample], AudioBufferError>;

    /// Returns an iterator over each channel in the audio buffer.
    /// Iterates from the 1st channel (channel `0`) to the highest.
    fn channels_iter(&self) -> Result<impl Iterator<Item = &[Sample]>, AudioBufferError>;

    /// Returns the single channel's samples.
    ///
    /// Should return `Err(NotMono)` if `channels != 1`.
    fn mono(&self) -> Result<&[Sample], AudioBufferError>;

    /// Returns the internal audio buffer as a contiguous slice,
    /// without checking the number of channels it contains.
    fn as_slice(&self) -> &[Sample];
}
