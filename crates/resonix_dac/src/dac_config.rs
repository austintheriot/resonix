use cpal::{
    traits::{DeviceTrait, HostTrait},
    Device, Host, SampleFormat, StreamConfig,
};

use crate::DACConfigBuildError;

pub struct DACConfig {
    #[allow(unused)]
    pub(crate) host: Host,
    pub(crate) device: Device,
    #[allow(unused)]
    pub(crate) sample_format: SampleFormat,
    pub(crate) stream_config: StreamConfig,
}

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
