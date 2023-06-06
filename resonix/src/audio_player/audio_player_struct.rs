use std::sync::Arc;

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BuildStreamError, OutputCallbackInfo, Sample, Stream, StreamConfig,
};

use crate::{AudioConfig, AudioPlayerContext, WriteFrameToBuffer};

/// Creates an audios stream and returns it, along with the
/// audio configuration that was chosen.
pub struct AudioPlayer<D> {
    pub context: Arc<AudioPlayerContext<D>>,
    pub stream: Stream,
}

// todo: refactor to do all setup through an Enum and/or builder pattern
// rather than calling different setup functions

impl<UserData> AudioPlayer<UserData>
where
    UserData: Send + Sync + Sync + 'static,
{
    pub async fn from_audio_defaults_and_user_data<S, Callback, ExtractedData>(
        write_frame_to_buffer: Callback,
        user_data: UserData,
    ) -> Result<Self, BuildStreamError>
    where
        S: Sample,
        Callback: WriteFrameToBuffer<S, UserData, ExtractedData> + Send + Sync + 'static,
    {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("failed to find a default output device");
        let config = device.default_output_config().unwrap();
        let sample_format = config.sample_format();
        let stream_config: StreamConfig = config.into();
        let audio_config = AudioConfig {
            host,
            device,
            sample_format,
            stream_config,
        };

        Self::from_audio_config_and_user_data(audio_config, write_frame_to_buffer, user_data).await
    }

    pub async fn from_audio_config_and_user_data<S, Callback, ExtractedData>(
        audio_config: AudioConfig,
        write_frame_to_buffer: Callback,
        user_data: UserData,
    ) -> Result<Self, BuildStreamError>
    where
        S: Sample,
        Callback: WriteFrameToBuffer<S, UserData, ExtractedData> + Send + Sync + 'static,
    {
        let context = Arc::new(AudioPlayerContext {
            host: audio_config.host,
            device: audio_config.device,
            sample_format: audio_config.sample_format,
            stream_config: audio_config.stream_config,
            user_data,
        });

        let stream = Self::create_stream::<S, Callback, ExtractedData>(
            Arc::clone(&context),
            write_frame_to_buffer,
        )
        .await?;

        stream.play().unwrap();

        Ok(Self { context, stream })
    }

    async fn create_stream<S, Callback, ExtractedData>(
        context: Arc<AudioPlayerContext<UserData>>,
        mut write_frame_to_buffer: Callback,
    ) -> Result<Stream, BuildStreamError>
    where
        S: Sample,
        Callback: WriteFrameToBuffer<S, UserData, ExtractedData> + Send + Sync + 'static,
    {
        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
        let device = &context.device;
        let stream_config = &context.stream_config;
        let context = Arc::clone(&context);

        device.build_output_stream(
            stream_config,
            move |buffer: &mut [S], _: &OutputCallbackInfo| {
                write_frame_to_buffer.call(buffer, Arc::clone(&context))
            },
            err_fn,
        )
    }
}

impl AudioPlayer<()> {
    /// Creates audio player using default audio settings and
    /// does not have any user context associated with it.
    pub async fn from_audio_defaults<S, Callback, ExtractedData>(
        write_frame_to_buffer: Callback,
    ) -> Result<Self, BuildStreamError>
    where
        S: Sample,
        Callback: WriteFrameToBuffer<S, (), ExtractedData> + Send + Sync + 'static,
    {
        Self::from_audio_defaults_and_user_data(write_frame_to_buffer, ()).await
    }

    /// Creates audio player using specified audio config and
    /// does not have any user context associated with it.
    pub async fn from_audio_config<S, Callback, ExtractedData>(
        audio_config: AudioConfig,
        write_frame_to_buffer: Callback,
    ) -> Result<Self, BuildStreamError>
    where
        S: Sample,
        Callback: WriteFrameToBuffer<S, (), ExtractedData> + Send + Sync + 'static,
    {
        Self::from_audio_config_and_user_data(audio_config, write_frame_to_buffer, ()).await
    }
}

#[cfg(test)]
#[cfg_attr(feature = "ci", ignore)]
mod player_tests {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use crate::{AudioPlayer, AudioPlayerContext, AudioPlayerContextArg, UserDataFromContext};

    #[tokio::test]
    async fn calls_get_frame_closure() {
        let called = Arc::new(Mutex::new(false));

        let player = {
            let called = Arc::clone(&called);
            AudioPlayer::from_audio_defaults(move |_: &mut [f32]| {
                *called.lock().unwrap() = true;
            })
        }
        .await;

        std::thread::sleep(Duration::from_millis(1));

        assert!(player.is_ok());
        assert!(*called.lock().unwrap());
    }

    #[tokio::test]
    async fn allows_getting_owned_user_data_from_context() {
        struct UserData {
            example: String,
        }

        #[derive(Debug, PartialEq, Clone)]
        struct Example(String);

        impl UserDataFromContext<UserData> for Example {
            fn from_context(context: Arc<AudioPlayerContext<UserData>>) -> Self {
                Self(context.user_data.example.clone())
            }
        }

        let user_data = UserData {
            example: String::from("example"),
        };

        let called = Arc::new(Mutex::new(false));

        let player = {
            let called = Arc::clone(&called);
            AudioPlayer::from_audio_defaults_and_user_data(
                move |_: &mut [f32], example: Example| {
                    *called.lock().unwrap() = true;
                    assert_eq!(example, Example(String::from("example")))
                },
                user_data,
            )
        }
        .await;

        std::thread::sleep(Duration::from_millis(1));

        assert!(player.is_ok());

        assert!(*called.lock().unwrap());
    }

    #[tokio::test]
    async fn allows_getting_context_itself_as_arg() {
        struct UserData {
            example: String,
        }

        let user_data = UserData {
            example: String::from("example"),
        };

        let called = Arc::new(Mutex::new(false));

        let player = {
            let called = Arc::clone(&called);
            AudioPlayer::from_audio_defaults_and_user_data(
                move |_: &'_ mut [f32], context: AudioPlayerContextArg<UserData>| {
                    *called.lock().unwrap() = true;
                    // possible to borrow inner context values here
                    assert_eq!(&context.user_data.example, "example");
                },
                user_data,
            )
        }
        .await;

        std::thread::sleep(Duration::from_millis(1));

        assert!(player.is_ok());
        assert!(*called.lock().unwrap());
    }

    #[tokio::test]
    async fn allows_combining_different_arguments() {
        struct UserData {
            a: String,
            b: u32,
            c: f64,
            d: Vec<f32>,
            called: Arc<Mutex<bool>>,
        }

        impl UserDataFromContext<UserData> for String {
            fn from_context(context: Arc<AudioPlayerContext<UserData>>) -> Self {
                context.user_data.a.clone()
            }
        }

        impl UserDataFromContext<UserData> for u32 {
            fn from_context(context: Arc<AudioPlayerContext<UserData>>) -> Self {
                context.user_data.b
            }
        }

        impl UserDataFromContext<UserData> for f64 {
            fn from_context(context: Arc<AudioPlayerContext<UserData>>) -> Self {
                context.user_data.c
            }
        }

        impl UserDataFromContext<UserData> for Vec<f32> {
            fn from_context(context: Arc<AudioPlayerContext<UserData>>) -> Self {
                context.user_data.d.clone()
            }
        }

        let called = Arc::new(Mutex::new(false));

        let user_data = UserData {
            a: String::from("example"),
            b: 1,
            c: 2.0,
            d: vec![1.0, 2.0, 3.0],
            called: Arc::clone(&called),
        };

        let player = {
            AudioPlayer::from_audio_defaults_and_user_data(
                move |_: &'_ mut [f32],
                      context: AudioPlayerContextArg<UserData>,
                      string: String,
                      uint: u32,
                      float: f64,
                      vec: Vec<f32>| {
                    *context.user_data.called.lock().unwrap() = true;
                    assert_eq!(string, String::from("example"));
                    assert_eq!(uint, 1);
                    assert_eq!(float, 2.0);
                    assert_eq!(vec, vec![1.0, 2.0, 3.0]);
                },
                user_data,
            )
        }
        .await;

        std::thread::sleep(Duration::from_millis(1));

        assert!(player.is_ok());
        assert!(*called.lock().unwrap());
    }
}
