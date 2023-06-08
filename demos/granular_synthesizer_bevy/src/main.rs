use std::{sync::Arc, time::Duration};

use bevy::prelude::*;
use resonix::{
    downmix_panning_to_buffer, AudioOut as ResonixAudioOut, AudioOutConfig,
    GranularSynthesizer as ResonixGranularSynthesizer, GranularSynthesizerAction,
};
use rodio::{
    cpal::{self, traits::HostTrait, StreamConfig},
    Decoder, DeviceTrait, Source,
};

#[derive(Component)]
struct GranularSynthesizer(ResonixGranularSynthesizer);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_audio)
        .run();
}

pub fn i16_array_to_f32(data: Vec<i16>) -> Vec<f32> {
    data.into_iter().map(|el| el as f32 / 65536.0).collect()
}

/// Converts default mp3 file to raw audio sample data
fn load_default_buffer() -> Arc<Vec<f32>> {
    // get audio file data at compile time
    let audio_file_slice =
        std::io::Cursor::new(include_bytes!("../../../assets/ecce_nova_3.mp3").as_ref());
    let mp3_source = Decoder::new(audio_file_slice).unwrap();
    let num_channels = mp3_source.channels() as usize;
    let mp3_source_data: Vec<f32> = i16_array_to_f32(mp3_source.collect());
    let left_channel_audio_data = mp3_source_data.into_iter().step_by(num_channels).collect();

    Arc::new(left_channel_audio_data)
}

fn setup_audio(world: &mut World) {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let config = device.default_output_config().unwrap();
    let sample_format = config.sample_format();
    let stream_config: StreamConfig = config.into();
    let audio_out_config = AudioOutConfig {
        device,
        host,
        sample_format,
        stream_config,
    };
    let output_sample_rate = audio_out_config.stream_config.sample_rate.0;
    let output_num_channels = audio_out_config.stream_config.channels as usize;

    // writing audio buffer data into a single vec here prevents lots
    // of wasted time on unnecessary allocations - any initial number is fine
    // here, since it will get resized to match current number of audio channels
    // in the frame
    let mut frame_buffer_data = vec![0.0; 0];

    let mut downmixed_frame_buffer_data = vec![0.0; output_num_channels];

    // Holds frame data before it is copied into the actual audio output buffer--
    // holding this data in a temporary buffer makes copying the data when recording simpler
    let mut final_frame_values = vec![0.0; output_num_channels];

    let mut resonix_granular_synthesizer = ResonixGranularSynthesizer::new();
    resonix_granular_synthesizer
        .set_num_channels(250)
        .set_buffer(load_default_buffer())
        .set_grain_len(Duration::from_millis(1000))
        .set_sample_rate(output_sample_rate);

    let write_frame_to_buffer = move |output_frame_buffer: &mut [f32]| {
        for output_channel_sample in output_frame_buffer.chunks_mut(output_num_channels) {
            // get next frame from granular synth
            resonix_granular_synthesizer.next_frame_into_buffer(&mut frame_buffer_data);

            // mix multi-channel down to number of outputs
            downmix_panning_to_buffer(
                &mut frame_buffer_data,
                output_num_channels as u32,
                &mut downmixed_frame_buffer_data,
            );

            const GAIN: f32 = 1.0;

            // gate final output with global gain
            final_frame_values
                .iter_mut()
                .zip(downmixed_frame_buffer_data.iter())
                .for_each(|(result, &sample)| {
                    *result = sample * GAIN;
                });

            for (i, sample) in output_channel_sample.iter_mut().enumerate() {
                *sample = cpal::Sample::from::<f32>(&final_frame_values[i]);
            }
        }
    };

    let audio_out = futures::executor::block_on(ResonixAudioOut::from_audio_out_config(
        audio_out_config,
        write_frame_to_buffer,
    ))
    .expect("Error initializing audio player");

    world.insert_non_send_resource(audio_out);
}
