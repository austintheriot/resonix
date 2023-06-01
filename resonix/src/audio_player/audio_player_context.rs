use cpal::{Device, Host, SampleFormat, StreamConfig};

/// Audio data
pub struct AudioPlayerContext {
    pub host: Host,
    pub device: Device,
    pub sample_format: SampleFormat,
    pub stream_config: StreamConfig,
}
