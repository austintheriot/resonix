use audio::{
    downmix_simple_to_buffer, granular_synthesizer::GranularSynthesizer,
    granular_synthesizer_action::GranularSynthesizerAction,
};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Stream, StreamConfig,
};
use rodio::{Decoder, Source};
use std::{
    rc::Rc,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::time::sleep;

/// Converts default mp3 file to raw audio sample data
fn load_default_buffer() -> Arc<Vec<f32>> {
    // get audio file data as compile time
    let audio_file_slice =
        std::io::Cursor::new(include_bytes!("../../../assets/ecce_nova_3.mp3").as_ref());
    let mp3_source = Decoder::new(audio_file_slice).unwrap();
    let num_channels = mp3_source.channels() as usize;
    let mp3_source_data: Vec<f32> =
        granular_synthesizer_cli::utils::i16_array_to_f32(mp3_source.collect());
    let left_channel_audio_data = mp3_source_data.into_iter().step_by(num_channels).collect();

    Arc::new(left_channel_audio_data)
}

/// This function is called periodically to write audio data into an audio output buffer
fn write_data_to_frame_buffer<T>(
    output: &mut [T],
    channels: usize,
    write_frame_values_to_buffer: &mut dyn FnMut(&mut Vec<f32>),
    final_frame_values: &mut Vec<f32>,
) where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        write_frame_values_to_buffer(final_frame_values);

        for (i, sample) in frame.iter_mut().enumerate() {
            *sample = cpal::Sample::from::<f32>(&final_frame_values[i]);
        }
    }
}

/// Setup all audio data and processes and begin playing
pub async fn run<T>(
    device: &cpal::Device,
    stream_config: &cpal::StreamConfig,
    granular_synthesizer: Arc<Mutex<GranularSynthesizer>>,
) -> Result<Stream, anyhow::Error>
where
    T: cpal::Sample,
{
    // this is the config of the output audio
    let output_sample_rate = stream_config.sample_rate.0;
    let output_num_channels = stream_config.channels as usize;
    let max_num_channels = 50;

    {
        // configure granular synthesizer settings
        let mut granular_synthesizer_lock = granular_synthesizer.lock().unwrap();
        granular_synthesizer_lock.set_buffer(load_default_buffer());
        granular_synthesizer_lock.set_grain_len(Duration::from_millis(1000));
        granular_synthesizer_lock.set_num_channels(250);
        granular_synthesizer_lock.set_sample_rate(output_sample_rate);
        granular_synthesizer_lock
            .set_selection_start(0.0)
            .set_selection_end(1.0);
    }

    // writing audio buffer data into a single vec here prevents lots
    // of wasted time on unnecessary allocations - any initial number is fine
    // here, since it will get resized to match current number of audio channels
    // in the frame
    let mut generated_granular_synth_frame = vec![0.0; max_num_channels as usize];

    // writing audio buffer data into a single vec here prevents lots
    // of wasted time on unnecessary allocations
    let mut final_frame_values = vec![0.0; max_num_channels as usize];

    // Called for every audio frame to generate appropriate sample
    let mut write_frame_values_to_buffer = move |output_frame_buffer: &mut Vec<f32>| {
        // get next frame from granular synth
        let mut granular_synthesizer_lock = granular_synthesizer.lock().unwrap();
        granular_synthesizer_lock.next_frame_into_buffer(&mut generated_granular_synth_frame);

        // mix multi-channel down to number of outputs
        downmix_simple_to_buffer(
            &generated_granular_synth_frame,
            output_num_channels as u32,
            output_frame_buffer,
        );
    };

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        stream_config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data_to_frame_buffer(
                data,
                output_num_channels,
                &mut write_frame_values_to_buffer,
                &mut final_frame_values,
            )
        },
        err_fn,
    )?;

    stream.play()?;

    Ok(stream)
}

#[tokio::main]
pub async fn main() {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let config = device.default_output_config().unwrap();
    let sample_format = config.sample_format();
    let stream_config: StreamConfig = config.into();

    let granular_synthesizer = Arc::new(Mutex::new(GranularSynthesizer::new()));

    let _stream_handle = Rc::new(match sample_format {
        cpal::SampleFormat::F32 => run::<f32>(&device, &stream_config, granular_synthesizer)
            .await
            .unwrap(),
        cpal::SampleFormat::I16 => run::<i16>(&device, &stream_config, granular_synthesizer)
            .await
            .unwrap(),
        cpal::SampleFormat::U16 => run::<u16>(&device, &stream_config, granular_synthesizer)
            .await
            .unwrap(),
    });

    sleep(Duration::from_secs(u64::MAX)).await;
}
