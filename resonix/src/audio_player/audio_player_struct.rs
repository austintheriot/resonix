use std::sync::Arc;

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BuildStreamError, OutputCallbackInfo, Sample, Stream, StreamConfig,
};

use crate::{AudioPlayerContext, GetFrame};

pub struct AudioPlayer<D> {
    context: Arc<AudioPlayerContext<D>>,
    stream: Stream,
}

impl<UserData> AudioPlayer<UserData>
where
    UserData: Send + Sync + 'static,
{
    pub async fn from_defaults_and_user_context<S, Callback, ExtractedData>(
        get_frame: Callback,
        data: UserData,
    ) -> Result<Self, BuildStreamError>
    where
        S: Sample,
        Callback: GetFrame<S, UserData, ExtractedData> + Send + 'static + Clone,
    {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("failed to find a default output device");
        let config = device.default_output_config().unwrap();
        let sample_format = config.sample_format();
        let stream_config: StreamConfig = config.into();

        let context = Arc::new(AudioPlayerContext {
            host,
            device,
            sample_format,
            stream_config,
            data,
        });

        let stream =
            Self::run::<S, Callback, ExtractedData>(Arc::clone(&context), get_frame).await?;

        stream.play().unwrap();

        Ok(Self { context, stream })
    }

    async fn run<S, Callback, ExtractedData>(
        context: Arc<AudioPlayerContext<UserData>>,
        get_frame: Callback,
    ) -> Result<Stream, BuildStreamError>
    where
        S: Sample,
        Callback: GetFrame<S, UserData, ExtractedData> + Send + 'static + Clone,
    {
        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
        let device = &context.device;
        let stream_config = &context.stream_config;
        let context = Arc::clone(&context);

        device.build_output_stream(
            stream_config,
            move |buffer: &mut [S], _: &OutputCallbackInfo| {
                get_frame.clone().call(buffer, &context)
            },
            err_fn,
        )
    }
}

impl AudioPlayer<()> {
    /// Creates audio player that does not have any user context
    /// associated with it.
    pub async fn from_defaults<S, Callback, ExtractedData>(
        get_frame: Callback,
    ) -> Result<Self, BuildStreamError>
    where
        S: Sample,
        Callback: for<'a> GetFrame<S, (), ExtractedData> + Send + 'static + Clone,
    {
        Self::from_defaults_and_user_context(get_frame, ()).await
    }
}

#[cfg(test)]
mod player_tests {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use crate::{AudioPlayer, AudioPlayerContext, FromContext};

    #[tokio::test]
    async fn calls_get_frame_closure() {
        let called = Arc::new(Mutex::new(false));

        let player = {
            let called = Arc::clone(&called);
            AudioPlayer::from_defaults(move |_: &mut [f32]| {
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

        impl<'a> FromContext<'a, UserData> for Example {
            fn from_context<'b: 'a>(context: &'b AudioPlayerContext<UserData>) -> Self {
                Self(context.data.example.clone())
            }
        }

        let data = UserData {
            example: String::from("example"),
        };

        let called = Arc::new(Mutex::new(false));

        let player = {
            let called = Arc::clone(&called);
            AudioPlayer::from_defaults_and_user_context(
                move |_: &mut [f32], example: Example| {
                    *called.lock().unwrap() = true;
                    assert_eq!(example, Example(String::from("example")))
                },
                data,
            )
        }
        .await;

        std::thread::sleep(Duration::from_millis(1));

        assert!(player.is_ok());

        assert!(*called.lock().unwrap());
    }

    #[tokio::test]
    async fn allows_getting_user_data_reference_from_context() {
        struct UserData {
            example: String,
        }

        #[derive(Debug, PartialEq, Clone)]
        struct Example<'a>(&'a str);

        impl<'a> FromContext<'a, UserData> for Example<'a> {
            fn from_context<'b: 'a>(context: &'b AudioPlayerContext<UserData>) -> Self {
                Self(&context.data.example)
            }
        }

        let data = UserData {
            example: String::from("example"),
        };

        let called = Arc::new(Mutex::new(false));

        let player = {
            let called = Arc::clone(&called);
            AudioPlayer::from_defaults_and_user_context(
                move |_: &mut [f32], example: Example| {
                    *called.lock().unwrap() = true;
                    assert_eq!(example, "example");
                },
                data,
            )
        }
        .await;

        std::thread::sleep(Duration::from_millis(1));

        assert!(player.is_ok());
        assert!(*called.lock().unwrap());
    }
}
