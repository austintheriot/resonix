use cpal::{Device, Host, SampleFormat, StreamConfig};

pub struct AudioOutConfig {
    pub host: Host,
    pub device: Device,
    pub sample_format: SampleFormat,
    pub stream_config: StreamConfig,
}
