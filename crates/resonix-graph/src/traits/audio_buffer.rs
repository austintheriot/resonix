use crate::primitives::{AudioBufferError, Channel, Sample};

pub trait AudioBuffer {
    fn block_size(&self) -> usize;

    fn channels(&self) -> usize;

    /// Returns the samples for channel `c` (0-indexed).
    ///
    /// Should return `Err(ChannelOutOfRange)` if `c >= self.channels()`.
    fn channel(&self, channel: impl Into<Channel>) -> Result<&[Sample], AudioBufferError>;

    fn channels_iter(&self) -> Result<impl Iterator<Item = &[Sample]>, AudioBufferError>;

    /// Returns the single channel's samples.
    ///
    /// Should return `Err(NotMono)` if `channels != 1`.
    fn mono(&self) -> Result<&[Sample], AudioBufferError>;
}
