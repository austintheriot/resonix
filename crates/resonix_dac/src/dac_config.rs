#[cfg(not(test))]
use cpal::{
    traits::{DeviceTrait, HostTrait},
    DefaultStreamConfigError, Device, Host, SampleFormat, StreamConfig,
};

#[cfg(not(test))]
use thiserror::Error;

#[cfg(test)]
use std::sync::Mutex;

use crate::DataFromDACConfig;
use std::sync::Arc;

#[cfg(not(test))]
pub struct DACConfig {
    pub host: Host,
    pub device: Device,
    pub sample_format: SampleFormat,
    pub stream_config: StreamConfig,
}

/// when testing, mock functionality
#[cfg(test)]
pub struct DACConfig {
    pub data_written: Arc<Mutex<Vec<f32>>>,
}

#[cfg(not(test))]
#[derive(Error, Debug)]
pub enum DACConfigBuildError {
    #[error("no audio output devices found")]
    NooOutputDevicesAvailable,
    #[error("no default stream config available. original error: {0:?}")]
    DefaultStreamConfigError(#[from] DefaultStreamConfigError),
}

#[cfg(not(test))]
impl DACConfig {
    #[cfg(test)]
    pub fn from_defaults() -> Result<Self, DACConfigBuildError> {
        Ok(Self)
    }

    #[cfg(not(test))]
    pub fn from_defaults() -> Result<Self, DACConfigBuildError> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(DACConfigBuildError::NooOutputDevicesAvailable)?;
        let config = device.default_output_config()?;
        let sample_format = config.sample_format();
        let stream_config: StreamConfig = config.into();

        Ok(Self {
            host,
            device,
            sample_format,
            stream_config,
        })
    }
}

impl DataFromDACConfig for Arc<DACConfig> {
    fn from_config(config: Arc<DACConfig>) -> Self {
        Arc::clone(&config)
    }
}
