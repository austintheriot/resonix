use std::{fmt::Debug, sync::Arc};

use cpal::{
    traits::{DeviceTrait, HostTrait},
    BuildStreamError, DefaultStreamConfigError, OutputCallbackInfo, PlayStreamError, Sample,
    Stream, StreamConfig,
};
use thiserror::Error;

use crate::{DACConfig, DACConfigBuildError, WriteFrameToBuffer};

#[derive(Error, Debug)]
pub enum DACBuildError {
    #[error("Failed to build stream. original error: {0:?}")]
    Disconnect(#[from] BuildStreamError),
    #[error("No audio output devices found")]
    NooOutputDevicesAvailable,
    #[error("No default stream config available. original error: {0:?}")]
    DefaultStreamConfigError(#[from] DefaultStreamConfigError),
    #[error("Could not play stream. original error: {0:?}")]
    PlayStreamError(#[from] PlayStreamError),
    #[error("Could not create DACConfig. original error: {0:?}")]
    DACConfigBuildError(#[from] DACConfigBuildError),
}

/// Creates an audios stream and returns it, along with the
/// audio configuration that was chosen.
pub struct DAC {
    pub config: Arc<DACConfig>,
    pub stream: Stream,
}

// todo: refactor to do all setup through an Enum and/or builder pattern
// rather than calling different setup functions

impl DAC {
    pub async fn from_dac_defaults<S, Callback, ExtractedData>(
        write_frame_to_buffer: Callback,
    ) -> Result<Self, DACBuildError>
    where
        S: Sample,
        Callback: WriteFrameToBuffer<S, ExtractedData> + Send + Sync + 'static,
    {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(DACBuildError::NooOutputDevicesAvailable)?;
        let config = device.default_output_config()?;
        let sample_format = config.sample_format();
        let stream_config: StreamConfig = config.into();

        let dac_config = DACConfig {
            host,
            device,
            sample_format,
            stream_config,
        };

        Self::from_dac_config(dac_config, write_frame_to_buffer).await
    }

    pub async fn from_dac_config<S, Callback, ExtractedData>(
        dac_config: DACConfig,
        write_frame_to_buffer: Callback,
    ) -> Result<Self, DACBuildError>
    where
        S: Sample,
        Callback: WriteFrameToBuffer<S, ExtractedData> + Send + Sync + 'static,
    {
        let config = Arc::new(dac_config);

        let stream = Self::create_stream::<S, Callback, ExtractedData>(
            Arc::clone(&config),
            write_frame_to_buffer,
        )
        .await?;

        Ok(Self { config, stream })
    }

    async fn create_stream<S, Callback, ExtractedData>(
        config: Arc<DACConfig>,
        mut write_frame_to_buffer: Callback,
    ) -> Result<Stream, BuildStreamError>
    where
        S: Sample,
        Callback: WriteFrameToBuffer<S, ExtractedData> + Send + Sync + 'static,
    {
        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
        let device = &config.device;
        let stream_config = &config.stream_config;
        let config = Arc::clone(&config);

        device.build_output_stream(
            stream_config,
            move |buffer: &mut [S], _: &OutputCallbackInfo| {
                write_frame_to_buffer.call(buffer, Arc::clone(&config))
            },
            err_fn,
        )
    }
}

impl Debug for DAC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DAC")
            .field("config", &"[native code]")
            .field("stream", &"[native code]")
            .finish()
    }
}

#[cfg(test)]
mod audio_out_tests {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use crate::{DACConfig, DAC};

    #[tokio::test]
    async fn calls_get_frame_closure() {
        let called = Arc::new(Mutex::new(false));

        let player = {
            let called = Arc::clone(&called);
            DAC::from_dac_defaults(move |_: &mut [f32]| {
                *called.lock().unwrap() = true;
            })
        }
        .await;

        std::thread::sleep(Duration::from_millis(1000));

        assert!(player.is_ok());
        assert!(*called.lock().unwrap());
    }

    #[tokio::test]
    async fn allows_getting_config_itself_as_arg() {
        let called = Arc::new(Mutex::new(false));

        let player = {
            let called = Arc::clone(&called);
            DAC::from_dac_defaults(move |_buffer: &'_ mut [f32], _config: Arc<DACConfig>| {
                *called.lock().unwrap() = true;
            })
        }
        .await;

        assert!(player.is_ok());

        std::thread::sleep(Duration::from_millis(1000));
        assert!(*called.lock().unwrap());
    }
}
