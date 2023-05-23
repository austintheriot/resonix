use audio::{
    downmix_simple, granular_synthesizer::GranularSynthesizer,
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
    let mp3_source_data: Vec<f32> = cli::utils::i16_array_to_f32(mp3_source.collect());
    let left_channel_audio_data = mp3_source_data.into_iter().step_by(num_channels).collect();

    Arc::new(left_channel_audio_data)
}

/// This function is called periodically to write audio data into an audio output buffer
fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> Vec<f32>)
where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        let output_samples = next_sample();

        for (i, sample) in frame.iter_mut().enumerate() {
            *sample = cpal::Sample::from::<f32>(&output_samples[i]);
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
        granular_synthesizer_lock.set_density(1.0);
        granular_synthesizer_lock.set_grain_len_max(1.0);
        granular_synthesizer_lock.set_grain_len_min(0.0);
        granular_synthesizer_lock.set_max_number_of_channels(max_num_channels);
        granular_synthesizer_lock.set_sample_rate(output_sample_rate);
        granular_synthesizer_lock
            .set_selection_start(0.0)
            .set_selection_end(1.0);
    }

    // writing audio buffer data into a single vec here prevents lots
    // of wasted time on unnecessary allocations
    let mut frame_buffer_data = vec![0.0; max_num_channels as usize];

    // Called for every audio frame to generate appropriate sample
    let mut next_value = move || {
        // get next frame from granular synth
        let mut granular_synthesizer_lock = granular_synthesizer.lock().unwrap();
        let frame = granular_synthesizer_lock.next_frame_into_buffer(&mut frame_buffer_data);

        // mix multi-channel down to number of outputs
        downmix_simple(frame, output_num_channels as u32)
    };

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        stream_config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, output_num_channels, &mut next_value)
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
