use std::sync::Arc;

use cpal::{Device, Host, SampleFormat, StreamConfig};

use crate::DataFromDACConfig;

pub struct DACConfig {
    pub host: Host,
    pub device: Device,
    pub sample_format: SampleFormat,
    pub stream_config: StreamConfig,
}

impl DataFromDACConfig for Arc<DACConfig> {
    fn from_config(config: Arc<DACConfig>) -> Self {
        Arc::clone(&config)
    }
}
