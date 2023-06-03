use cpal::{Device, Host, SampleFormat, StreamConfig};

/// Audio data
pub struct AudioPlayerContext<D> {
    pub host: Host,
    pub device: Device,
    pub sample_format: SampleFormat,
    pub stream_config: StreamConfig,
    /// This is arbitrary user-specified data that the user can associate with
    /// the audio context, making it easily retrievable by
    /// implementing FromContext for U
    pub data: D,
}
