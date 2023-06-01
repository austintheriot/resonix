use std::sync::Arc;

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BuildStreamError, OutputCallbackInfo, Sample, Stream, StreamConfig,
};

use crate::{AudioPlayerContext, GetFrame};

pub struct AudioPlayer {
    context: Arc<AudioPlayerContext>,
    stream: Stream,
}

impl AudioPlayer {
    pub async fn from_default_host<S: Sample, T, G: GetFrame<S, T> + Send + 'static + Clone>(
        get_frame: G,
    ) -> Result<Self, BuildStreamError> {
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
        });

        let stream = Self::run::<S, T, G>(Arc::clone(&context), get_frame).await?;

        stream.play().unwrap();

        Ok(Self { context, stream })
    }

    async fn run<S: Sample, T, G: GetFrame<S, T> + Send + 'static + Clone>(
        context: Arc<AudioPlayerContext>,
        get_frame: G,
    ) -> Result<Stream, BuildStreamError> {
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

#[cfg(test)]
mod player_tests {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use crate::AudioPlayer;

    #[tokio::test]
    async fn calls_get_frame_closure() {
        let called = Arc::new(Mutex::new(false));

        let player = {
            let called = called.clone();
            AudioPlayer::from_default_host(move |_: &mut [f32]| {
                *called.lock().unwrap() = true;
            })
        }
        .await;

        std::thread::sleep(Duration::from_millis(1));

        assert!(player.is_ok());
        assert!(*called.lock().unwrap());
    }
}
