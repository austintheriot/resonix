use std::{fmt::Debug, sync::Arc};

#[cfg(not(test))]
use cpal::{
    traits::{DeviceTrait, HostTrait},
    Stream, StreamConfig,
};
use cpal::{BuildStreamError, DefaultStreamConfigError, PlayStreamError, Sample};
#[cfg(test)]
use std::any::Any;
#[cfg(test)]
use std::sync::Mutex;
use thiserror::Error;

#[cfg(test)]
use crate::{DACConfig, WriteFrameToBuffer};

#[cfg(not(test))]
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
    #[cfg(not(test))]
    #[error("Could not create DACConfig. original error: {0:?}")]
    DACConfigBuildError(#[from] DACConfigBuildError),
}

/// Creates an audios stream and returns it, along with the
/// audio configuration that was chosen.
pub struct DAC {
    // "actual" implementation fields:
    pub config: Arc<DACConfig>,
    #[cfg(not(test))]
    pub stream: Stream,

    // test-specific fields for mocking:
    /// must use `Any` for this type, since `!` has not been stabilized yet
    #[cfg(test)]
    pub join_handle: Box<dyn Any>,
    #[cfg(test)]
    pub data_written: Arc<Mutex<Vec<f32>>>,
}

// todo: refactor to do all setup through an Enum and/or builder pattern
// rather than calling different setup functions

impl DAC {
    pub fn from_dac_defaults<S, Callback, ExtractedData>(
        write_frame_to_buffer: Callback,
    ) -> Result<Self, DACBuildError>
    where
        S: Sample,
        Callback: WriteFrameToBuffer<S, ExtractedData> + Send + Sync + 'static,
    {
        #[cfg(not(test))]
        let dac_config = {
            let host = cpal::default_host();
            let device = host
                .default_output_device()
                .ok_or(DACBuildError::NooOutputDevicesAvailable)?;
            let config = device.default_output_config()?;
            let sample_format = config.sample_format();
            let stream_config: StreamConfig = config.into();

            DACConfig::new(host, device, sample_format, stream_config)
        };

        #[cfg(test)]
        let dac_config = DACConfig;

        Self::from_dac_config(dac_config, write_frame_to_buffer)
    }

    pub fn from_dac_config<S, Callback, ExtractedData>(
        dac_config: DACConfig,
        write_frame_to_buffer: Callback,
    ) -> Result<Self, DACBuildError>
    where
        S: Sample,
        Callback: WriteFrameToBuffer<S, ExtractedData> + Send + 'static,
    {
        let config = Arc::new(dac_config);
        #[cfg(not(test))]
        {
            let stream = Self::create_stream::<S, Callback, ExtractedData>(
                Arc::clone(&config),
                write_frame_to_buffer,
            )?;

            Ok(Self { config, stream })
        }

        #[cfg(test)]
        {
            let data_written = Arc::new(Mutex::new(Vec::new()));
            let handle = Self::create_mock_stream::<S, Callback, ExtractedData>(
                Arc::clone(&config),
                Arc::clone(&data_written),
                write_frame_to_buffer,
            )?;

            Ok(Self {
                config,
                join_handle: handle,
                data_written,
            })
        }
    }

    #[cfg(not(test))]
    fn create_stream<S, Callback, ExtractedData>(
        config: Arc<DACConfig>,
        mut write_frame_to_buffer: Callback,
    ) -> Result<Stream, BuildStreamError>
    where
        S: Sample,
        Callback: WriteFrameToBuffer<S, ExtractedData> + Send + 'static,
    {
        use cpal::OutputCallbackInfo;

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

    /// Mock calling from audio thread
    #[cfg(test)]
    fn create_mock_stream<S, Callback, ExtractedData>(
        config: Arc<DACConfig>,
        data_written: Arc<Mutex<Vec<f32>>>,
        mut write_frame_to_buffer: Callback,
    ) -> Result<Box<dyn Any>, BuildStreamError>
    where
        S: Sample,
        Callback: WriteFrameToBuffer<S, ExtractedData> + Send + 'static,
    {
        use std::time::Duration;

        Ok(Box::new(std::thread::spawn(move || loop {
            let num_frames = 1;
            let num_channels = 2;

            // new buffer to write data into
            let mut buffer = vec![S::from(&0.0); num_frames * num_channels];

            write_frame_to_buffer.call(buffer.as_mut_slice(), Arc::clone(&config));

            // copy into a stored buffer for testing against later
            data_written
                .lock()
                .unwrap()
                .extend(buffer.into_iter().map(|s| s.to_f32()));
            std::thread::sleep(Duration::from_millis(1))
        })) as Box<dyn Any>)
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
        };

        assert!(player.is_ok());

        let mut tries = 0;
        while !*called.lock().unwrap() {
            tries += 1;

            if tries >= 30 {
                panic!("Failed to call closure callback after {tries:?} tries");
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
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
        .unwrap();

        let mut tries = 0;
        while !*called.lock().unwrap() {
            tries += 1;

            if tries >= 30 {
                panic!("Failed to call closure callback after {tries:?} tries");
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        assert!(!player.data_written.lock().unwrap().is_empty())
    }

    #[tokio::test]
    async fn mock_implementation_saves_written_data() {
        let mut current_sample = 0.0;

        let player = {
            DAC::from_dac_defaults(move |buffer: &'_ mut [f32], config: Arc<DACConfig>| {
                // write basic sequential data to outgoing buffer
                let num_channels = config.num_channels();
                for frame in buffer.chunks_mut(num_channels as usize) {
                    for channel in frame.iter_mut() {
                        *channel = cpal::Sample::from::<f32>(&current_sample);
                    }

                    current_sample += 0.5;
                }
            })
        }
        .unwrap();

        tokio::time::sleep(Duration::from_secs(1)).await;

        let data_written = player.data_written.lock().unwrap();
        assert_eq!(data_written[0..6], [0.0, 0.0, 0.5, 0.5, 1.0, 1.0]);
    }
}
