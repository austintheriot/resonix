extern crate anyhow;
extern crate clap;
extern crate cpal;
extern crate common;

use common::{grain_sample::GrainSample, grain::Grain};
use clap::arg;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rand::prelude::*;
use rodio::Decoder;
use std::{fs::File};

#[derive(Debug)]
struct Opt {
    device: String,
}

impl Opt {
    fn from_args() -> Self {
        let app = clap::Command::new("beep").arg(arg!([DEVICE] "The audio device to use"));
        let matches = app.get_matches();
        let device = matches.value_of("DEVICE").unwrap_or("default").to_string();
        Opt { device }
    }
}

/// Called periodically to fill a buffer with data
fn write_data<T>(
    output_data: &mut [T],
    channels: usize,
    next_sample: &mut dyn FnMut() -> (f32, f32),
) where
    T: cpal::Sample,
{
    for frame in output_data.chunks_mut(channels) {
        let (left_sample, right_sample) = next_sample();
        let left_sample = cpal::Sample::from::<f32>(&left_sample);
        let right_sample = cpal::Sample::from::<f32>(&right_sample);

        // assume a 2-channel system and just map to evens and odds if there are more channels
        for (i, sample) in frame.iter_mut().enumerate() {
            if i % 2 == 0 {
                *sample = left_sample;
            } else {
                *sample = right_sample;
            }
        }
    }
}

/// Creates a linear ramp from 0.0 -> 1.0 -> 0.0
/// 
/// The `current_index` is assumed to range from 0.0 -> 1.0
fn generate_triangle_envelope_value_from_percent(current_index: f32) -> f32 {
    (((current_index - 0.5).abs() * -1.0) + 0.5) * 2.0
}

fn i16_array_to_f32(data: Vec<i16>) -> Vec<f32> {
    data.into_iter().map(|el| el as f32 / 65536.0).collect()
}

pub fn run_audio<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), anyhow::Error>
where
    T: cpal::Sample,
{
    const NUM_CHANNELS: usize = 100;
    const ENVELOPE_LEN_MS_MIN: f32 = 1.0;
    const ENVELOPE_LEN_MS_MAX: f32 = 100.0;

    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;
    let envelope_len_samples_min = (sample_rate / (1000.0 / ENVELOPE_LEN_MS_MIN)) as usize;
    let envelope_len_samples_max = (sample_rate / (1000.0 / ENVELOPE_LEN_MS_MAX)) as usize;

    // get audio file data
    let file = File::open("../audio/pater_emon.mp3")?;
    let mp3_source = Decoder::new(file).unwrap();
    let mp3_source_data: Vec<f32> = i16_array_to_f32(mp3_source.collect());

    // associates each grain's sample value with it's envelope value
    // instantiated here to prevent allocations during audio calculations
    let mut frame_samples_and_envelopes = Vec::with_capacity(NUM_CHANNELS);
    for _ in 0..NUM_CHANNELS {
        frame_samples_and_envelopes.push(GrainSample::default());
    }

    // keeps track of where each grain should be in the buffer
    let mut channels_grains: Vec<Grain> = Vec::with_capacity(NUM_CHANNELS);
    for _ in 0..NUM_CHANNELS {
        channels_grains.push(Grain::default());
    }

    // Called for every audio frame to generate appropriate sample
    let mut next_value = move || {
        let mut rng = rand::thread_rng();

        // grain length should not exceed max mp3 source data length
        debug_assert!(mp3_source_data.len() > envelope_len_samples_max);

        // create new grains for any that are finished
        for grain in channels_grains.iter_mut() {
            if grain.finished {
                let envolope_len_samples =
                    rng.gen_range(envelope_len_samples_min..envelope_len_samples_max);
                let max_index = mp3_source_data.len() - envolope_len_samples;
                let start_frame = rng.gen_range(0..max_index);
                let end_frame = start_frame + envolope_len_samples;

                debug_assert!(start_frame > 0);
                debug_assert!(end_frame > 0);
                debug_assert!(start_frame < mp3_source_data.len());
                debug_assert!(end_frame < mp3_source_data.len());

                let new_grain = Grain::new(start_frame, end_frame);
                *grain = new_grain;
            }
        }

        debug_assert_eq!(channels_grains.len(), NUM_CHANNELS);

        // get value of each grain's current index in the buffer for each channel
        channels_grains
            .iter_mut()
            .enumerate()
            .for_each(|(i, grain)| {
                debug_assert_eq!(grain.finished, false);

                let envelope_percent =
                    ((grain.current_frame - grain.start_frame) as f32) / (grain.len as f32);
                debug_assert!(envelope_percent >= 0.0, "{}", envelope_percent);
                debug_assert!(envelope_percent < 1.0, "{}", envelope_percent);

                let envelope_value =
                    generate_triangle_envelope_value_from_percent(envelope_percent);
                let frame_index = grain.current_frame;
                let sample_value = mp3_source_data[frame_index];

                frame_samples_and_envelopes[i].sample_value = sample_value;
                frame_samples_and_envelopes[i].envelope_value = envelope_value;

                grain.get_next_frame();
            });

        // mix frame channels down to 2 channels (spacialize from left to right)
        let mut left = 0.0;
        let mut right = 0.0;
        for (i, grain_sample) in frame_samples_and_envelopes.iter().enumerate() {
            // earlier indexes to later indexes == left to right spacialization
            let left_spatialization_percent =
                1.0 - (i as f32) / (frame_samples_and_envelopes.len() as f32);
            let right_spatialization_percent =
                (i as f32) / (frame_samples_and_envelopes.len() as f32);

            // division by 0 will happen below if num of channels is less than 2
            debug_assert!(NUM_CHANNELS >= 2);

            // logarithmically scaling the volume seems to work well for very large numbers of voices
            let left_value_to_add = (grain_sample.sample_value
                * grain_sample.envelope_value
                * left_spatialization_percent)
                / (NUM_CHANNELS as f32).log(2.0);
            let right_value_to_add = (grain_sample.sample_value
                * grain_sample.envelope_value
                * right_spatialization_percent)
                / (NUM_CHANNELS as f32).log(2.0);

            left += left_value_to_add;
            right += right_value_to_add;
        }

        (left, right)
    };

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
    )?;

    stream.play()?;

    // sleep indefinitely
    std::thread::sleep(std::time::Duration::MAX);

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    let host = cpal::default_host();

    let device = if opt.device == "default" {
        host.default_output_device()
    } else {
        host.output_devices()?
            .find(|x| x.name().map(|y| y == opt.device).unwrap_or(false))
    }
    .expect("failed to find output device");
    println!("Output device: {}", device.name()?);

    let config = device.default_output_config().unwrap();
    println!("Default output config: {:#?}", config);

    match config.sample_format() {
        cpal::SampleFormat::F32 => run_audio::<f32>(&device, &config.into()),
        cpal::SampleFormat::I16 => run_audio::<i16>(&device, &config.into()),
        cpal::SampleFormat::U16 => run_audio::<u16>(&device, &config.into()),
    }
}
