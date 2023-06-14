use std::sync::Arc;

use resonix::{cpal, AudioOut, AudioOutContext, Sine};

pub async fn set_up_audio_stream() -> AudioOut<()> {
    let mut sines = vec![Sine::new(); 0];
    AudioOut::from_audio_defaults(
        move |buffer: &mut [f32], context: Arc<AudioOutContext<()>>| {
            let num_channels = context.stream_config.channels as usize;
            sines.resize_with(num_channels, Sine::new);
            sines.iter_mut().for_each(|sine| {
                sine.set_frequency(440.0)
                    .set_sample_rate(context.stream_config.sample_rate);
            });

            for frame in buffer.chunks_mut(num_channels) {
                for (i, channel) in frame.iter_mut().enumerate() {
                    let sine = &mut sines[i];
                    *channel = cpal::Sample::from::<f32>(&sine.next_sample());
                }
            }
        },
    )
    .await
    .expect("error setting up audio")
}
