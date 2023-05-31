use super::{
    audio_output_action::AudioOutputAction, audio_recorder_handle::AudioRecorderHandle,
    buffer_selection_action::BufferSelectionAction, decode, gain_action::GainAction,
    play_status::PlayStatus, play_status_action::PlayStatusAction,
    recording_status::RecordingStatus, recording_status_action::RecordingStatusAction,
    recording_status_handle::RecordingStatusHandle,
};
use crate::{
    audio::stream_handle::StreamHandle,
    components::controls_select_buffer::DEFAULT_AUDIO_FILE,
    state::{app_action::AppAction, app_state::AppState},
};
use audio::{downmix_panning_to_buffer, granular_synthesizer::GranularSynthesizerAction};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    OutputCallbackInfo, Stream, StreamConfig,
};
use gloo_net::http::Request;
use std::sync::Arc;
use yew::UseReducerHandle;

/// Converts default mp3 file to raw audio sample data
async fn load_default_buffer(app_state_handle: UseReducerHandle<AppState>) -> Arc<Vec<f32>> {
    let audio_context =
        web_sys::AudioContext::new().expect("Browser should have AudioContext implemented");

    // audio files are copied into static director for web (same directory as source wasm file)
    // fetch a default audio file at initialization time
    let mp3_file_bytes = Request::get(&format!("./{}", DEFAULT_AUDIO_FILE))
        .send()
        .await
        .unwrap()
        .binary()
        .await
        .unwrap();

    let audio_buffer = decode::decode_bytes(&audio_context, &mp3_file_bytes)
        .await
        .unwrap();
    let mp3_source_data = Arc::new(audio_buffer.get_channel_data(0).unwrap());
    app_state_handle.dispatch(AppAction::SetBuffer(Arc::clone(&mp3_source_data)));

    mp3_source_data
}

/// This function is called periodically to write audio data into an audio output buffer
fn write_data_to_frame_buffer<T>(
    output_frame_buffer: &mut [T],
    output_num_channels: usize,
    write_frame_values_to_buffer: &mut dyn FnMut(&mut Vec<f32>),
    recording_status_handle: RecordingStatusHandle,
    mut audio_recorder_handle: AudioRecorderHandle,
    final_frame_values: &mut Vec<f32>,
) where
    T: cpal::Sample,
{
    for output_channel_sample in output_frame_buffer.chunks_mut(output_num_channels) {
        write_frame_values_to_buffer(final_frame_values);

        // clone audio data into a recording buffer
        let is_recording = recording_status_handle.get() == RecordingStatus::Recording;
        if is_recording {
            let output_samples = final_frame_values.clone();
            audio_recorder_handle.extend(output_samples)
        }

        for (i, sample) in output_channel_sample.iter_mut().enumerate() {
            *sample = cpal::Sample::from::<f32>(&final_frame_values[i]);
        }
    }
}

/// Setup all audio data and processes and begin playing
pub async fn run<T>(
    app_state_handle: UseReducerHandle<AppState>,
    device: &cpal::Device,
    stream_config: &cpal::StreamConfig,
) -> Result<Stream, anyhow::Error>
where
    T: cpal::Sample,
{
    // this is the config of the output audio
    let output_sample_rate = stream_config.sample_rate.0;
    let output_num_channels = stream_config.channels as usize;
    let num_frames_between_saving_snapshot = output_sample_rate / 120;

    // only load if buffer hasn't been loaded
    if app_state_handle.buffer_handle.get_data().is_empty() {
        load_default_buffer(app_state_handle.clone()).await;
    }

    let buffer_selection_handle = app_state_handle.buffer_selection_handle.clone();
    let gain_handle = app_state_handle.gain_handle.clone();
    let status = app_state_handle.play_status_handle.clone();
    let mut granular_synthesizer_handle = app_state_handle.granular_synthesizer_handle.clone();
    let mut audio_output_handle = app_state_handle.audio_output_handle.clone();

    // make sure granular synthesizer's internal state is current with audio context state
    granular_synthesizer_handle.set_sample_rate(output_sample_rate);

    // writing audio buffer data into a single vec here prevents lots
    // of wasted time on unnecessary allocations - any initial number is fine
    // here, since it will get resized to match current number of audio channels
    // in the frame
    let mut frame_buffer_data = vec![0.0; 0];

    let mut downmixed_frame_buffer_data = vec![0.0; output_num_channels];

    // Holds frame data before it is copied into the actual audio output buffer--
    // holding this data in a temporary buffer makes copying the data when recording simpler
    let mut final_frame_values = vec![0.0; output_num_channels];

    let mut frame_count: u32 = 0;

    // Called for every audio frame to generate appropriate sample
    let mut write_frame_values_to_buffer = move |output_frame_buffer: &mut Vec<f32>| {
        frame_count = frame_count.wrapping_add(1);

        // if paused, do not process any audio, just return silence
        if let PlayStatus::Pause = status.get() {
            output_frame_buffer.fill(0.0);
            return;
        }

        // always keep granular_synth up-to-date with buffer selection from UI
        let (selection_start, selection_end) = buffer_selection_handle.get_buffer_start_and_end();
        granular_synthesizer_handle
            .set_selection_start(selection_start)
            .set_selection_end(selection_end);

        // get next frame from granular synth
        let frame = granular_synthesizer_handle.next_frame_into_buffer(&mut frame_buffer_data);

        // copy up-to-date audio output information into context for
        // reference in audio output visualization
        // (only visualize 2-channel audio, for performance reasons)
        if frame_count % num_frames_between_saving_snapshot == 0 {
            audio_output_handle.add_frame(frame.clone());
        }

        // mix multi-channel down to number of outputs
        downmix_panning_to_buffer(
            frame,
            output_num_channels as u32,
            &mut downmixed_frame_buffer_data,
        );

        // gate final output with global gain
        let gain = gain_handle.get();
        output_frame_buffer
            .iter_mut()
            .zip(downmixed_frame_buffer_data.iter())
            .for_each(|(result, &sample)| {
                *result = sample * gain;
            });
    };

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let recording_status_handle = app_state_handle.recording_status_handle.clone();
    let audio_recorder_handle = app_state_handle.audio_recorder_handle.clone();

    let stream = device.build_output_stream(
        stream_config,
        move |output_frame_buffer: &mut [T], _: &OutputCallbackInfo| {
            write_data_to_frame_buffer(
                output_frame_buffer,
                output_num_channels,
                &mut write_frame_values_to_buffer,
                recording_status_handle.clone(),
                audio_recorder_handle.clone(),
                &mut final_frame_values,
            )
        },
        err_fn,
    )?;

    stream.play()?;

    Ok(stream)
}

pub async fn initialize_audio(app_state_handle: UseReducerHandle<AppState>) -> StreamHandle {
    app_state_handle.dispatch(AppAction::SetAudioInitialized(false));
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let config = device.default_output_config().unwrap();
    let sample_format = config.sample_format();
    let stream_config: StreamConfig = config.into();
    app_state_handle.dispatch(AppAction::SetSampleRate(stream_config.sample_rate.0));
    app_state_handle.dispatch(AppAction::SetNumChannels(stream_config.channels as u32));

    StreamHandle::new(match sample_format {
        cpal::SampleFormat::F32 => run::<f32>(app_state_handle, &device, &stream_config)
            .await
            .unwrap(),
        cpal::SampleFormat::I16 => run::<i16>(app_state_handle, &device, &stream_config)
            .await
            .unwrap(),
        cpal::SampleFormat::U16 => run::<u16>(app_state_handle, &device, &stream_config)
            .await
            .unwrap(),
    })
}
