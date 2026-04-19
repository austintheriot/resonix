use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AudioBufferError {
    #[error("Not enough channels. Found 0")]
    ZeroChannels,
    #[error("Length channel match. Length is {len} and channels is {channels}")]
    LengthChannelMismatch { len: usize, channels: usize },
    #[error("Channel out of range. Index is {index} and channels is {channels}")]
    ChannelOutOfRange { index: usize, channels: usize },
    #[error("Not mono. Buffer has {channels} channels.")]
    NotMono { channels: usize },
}
