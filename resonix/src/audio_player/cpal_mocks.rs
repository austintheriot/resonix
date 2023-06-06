use cpal::{
    traits::{DeviceTrait, HostTrait},
    BuildStreamError, DefaultStreamConfigError, DeviceNameError, OutputCallbackInfo, StreamConfig,
    SupportedStreamConfig, SupportedStreamConfigsError, SupportedStreamConfigRange,
};

#[derive(Default)]
#[cfg(test)]
pub(crate) struct MockDevices {
    finished: bool,
}

#[cfg(test)]
impl Iterator for MockDevices {
    type Item = MockDevice;

    fn next(&mut self) -> Option<Self::Item> {
        match self.finished {
            true => None,
            false => {
                self.finished = true;
                Some(MockDevice)
            }
        }
    }
}

#[cfg(test)]
pub(crate) struct MockHost;

#[cfg(test)]
impl HostTrait for MockHost {
    type Devices = MockDevices;

    type Device = MockDevice;

    fn is_available() -> bool {
        true
    }

    fn devices(&self) -> Result<Self::Devices, cpal::DevicesError> {
        Ok(MockDevices::default())
    }

    fn default_input_device(&self) -> Option<Self::Device> {
        Some(MockDevice)
    }

    fn default_output_device(&self) -> Option<Self::Device> {
        Some(MockDevice)
    }
}

#[cfg(test)]
struct InputConfigs {
    finished: bool,
}

#[cfg(test)]
impl Iterator for InputConfigs {
    type Item = SupportedStreamConfigRange;

    fn next(&mut self) -> Option<Self::Item> {
        match self.finished {
            true => None,
            false => {
                self.finished = true;
                Some(SupportedStreamConfigRange {
                    
                })
            }
        }
    }
}

#[cfg(test)]
struct OutputConfig;

#[cfg(test)]
pub(crate) struct MockDevice;

#[cfg(test)]
impl DeviceTrait for MockDevice {
    type SupportedInputConfigs = InputConfigs;

    type SupportedOutputConfigs = OutputConfig;

    type Stream = ();

    fn name(&self) -> Result<String, DeviceNameError> {
        Ok(String::from("MockDevice"))
    }

    fn supported_input_configs(
        &self,
    ) -> Result<Self::SupportedInputConfigs, SupportedStreamConfigsError> {
        Ok(())
    }

    fn supported_output_configs(
        &self,
    ) -> Result<Self::SupportedOutputConfigs, SupportedStreamConfigsError> {
        Ok(())
    }

    fn default_input_config(&self) -> Result<SupportedStreamConfig, DefaultStreamConfigError> {
        Ok(())
    }

    fn default_output_config(&self) -> Result<SupportedStreamConfig, DefaultStreamConfigError> {
        Ok(())
    }

    fn build_input_stream_raw<D, E>(
        &self,
        config: &StreamConfig,
        sample_format: cpal::SampleFormat,
        data_callback: D,
        error_callback: E,
    ) -> Result<Self::Stream, BuildStreamError>
    where
        D: FnMut(&cpal::Data, &cpal::InputCallbackInfo) + Send + 'static,
        E: FnMut(cpal::StreamError) + Send + 'static,
    {
        Ok(())
    }

    fn build_output_stream_raw<D, E>(
        &self,
        config: &StreamConfig,
        sample_format: cpal::SampleFormat,
        data_callback: D,
        error_callback: E,
    ) -> Result<Self::Stream, BuildStreamError>
    where
        D: FnMut(&mut cpal::Data, &OutputCallbackInfo) + Send + 'static,
        E: FnMut(cpal::StreamError) + Send + 'static,
    {
        Ok(())
    }
}

#[cfg(test)]
use crate::AudioPlayerError;

// todo: refactor to do all setup through an Enum and/or builder pattern
// rather than calling different setup functions
#[cfg(test)]
pub(crate) fn get_audio_defaults() -> Result<(impl HostTrait, impl DeviceTrait), AudioPlayerError> {
    Ok((MockHost, MockDevice))
}
