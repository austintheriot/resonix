#[cfg(not(all(test, feature = "mock_test_dac")))]
use cpal::{
    traits::{DeviceTrait, HostTrait},
    DefaultStreamConfigError, Device, Host, SampleFormat, StreamConfig,
};

#[cfg(not(all(test, feature = "mock_test_dac")))]
use thiserror::Error;

use crate::DataFromDACConfig;
use std::sync::Arc;

#[cfg(not(all(test, feature = "mock_test_dac")))]
pub struct DACConfig {
    #[allow(unused)]
    pub(crate) host: Host,
    pub(crate) device: Device,
    #[allow(unused)]
    pub(crate) sample_format: SampleFormat,
    pub(crate) stream_config: StreamConfig,
}

/// when testing, mock functionality
#[cfg(all(test, feature = "mock_test_dac"))]
pub struct DACConfig;

#[cfg(not(all(test, feature = "mock_test_dac")))]
#[derive(Error, Debug)]
pub enum DACConfigBuildError {
    #[error("no audio output devices found")]
    NooOutputDevicesAvailable,
    #[error("no default stream config available. original error: {0:?}")]
    DefaultStreamConfigError(#[from] DefaultStreamConfigError),
}

#[cfg(not(all(test, feature = "mock_test_dac")))]
impl DACConfig {
    pub fn new(
        host: Host,
        device: Device,
        sample_format: SampleFormat,
        stream_config: StreamConfig,
    ) -> Self {
        Self {
            host,
            device,
            sample_format,
            stream_config,
        }
    }

    pub fn num_channels(&self) -> u16 {
        self.stream_config.channels
    }

    pub fn sample_rate(&self) -> u32 {
        self.stream_config.sample_rate.0
    }

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

#[cfg(all(test, feature = "mock_test_dac"))]
impl DACConfig {
    pub fn from_defaults() -> Result<Self, ()> {
        Ok(Self)
    }

    pub fn num_channels(&self) -> u32 {
        2
    }

    pub fn sample_rate(&self) -> u32 {
        44100
    }
}

impl DataFromDACConfig for Arc<DACConfig> {
    fn from_config(config: Arc<DACConfig>) -> Self {
        Arc::clone(&config)
    }
}
