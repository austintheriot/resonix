use std::sync::Arc;

use resonix::{cpal, DACConfig, Sine, SineInterface, DAC};

pub async fn set_up_audio_stream() -> DAC {
    let mut sines = vec![Sine::new(); 0];
    DAC::from_dac_defaults(move |buffer: &mut [f32], context: Arc<DACConfig>| {
        let num_channels = context.num_channels() as usize;
        sines.resize_with(num_channels, Sine::new);
        sines.iter_mut().for_each(|sine| {
            sine.set_frequency(440.0)
                .set_sample_rate(context.sample_rate());
        });

        for frame in buffer.chunks_mut(num_channels) {
            for (i, channel) in frame.iter_mut().enumerate() {
                let sine = &mut sines[i];
                *channel = cpal::Sample::from::<f32>(&sine.next_sample());
            }
        }
    })
    .expect("error setting up audio")
}
