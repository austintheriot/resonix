#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioBufferError {
    ZeroChannels,
    LengthChannelMismatch { len: usize, channels: usize },
    ChannelOutOfRange { index: usize, channels: usize },
    NotMono { channels: usize },
}
