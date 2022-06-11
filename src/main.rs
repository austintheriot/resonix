extern crate anyhow;
extern crate clap;
extern crate cpal;

use clap::arg;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rand::prelude::*;
use rodio::Decoder;
use std::{f64::consts::PI, fs::File};

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

        for (i, sample) in frame.iter_mut().enumerate() {
            if i % 2 == 0 {
                *sample = left_sample;
            } else {
                *sample = right_sample;
            }
        }
    }
}

fn generate_sine(sample_tick: f64, frequency: f64, frequency_modulation: f64) -> f64 {
    (2.0 * PI * (frequency * sample_tick + frequency_modulation)).sin()
}

/// Generates only the top portion of a sine wave (good envelope for granular synthesis)
///
/// The sine index (`current_index`) is assumed to range from 0.0 -> 1.0
fn generate_sine_envelope(current_index: f32) -> f32 {
    (current_index * std::f32::consts::PI).sin()
}

fn i16_array_to_f32(data: Vec<i16>) -> Vec<f32> {
    data.into_iter().map(|el| el as f32 / 65536.0).collect()
}

/// Generates a sine envolope (a Vec) with a length of `len`
fn generate_envolope_of_len(len: usize) -> Vec<f32> {
    let mut envelope_data = Vec::with_capacity(len);
    for i in 0..len {
        let percent_complete = i as f32 / len as f32;
        envelope_data.push(generate_sine_envelope(percent_complete));
    }
    envelope_data
}

pub fn run<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
) -> Result<(), anyhow::Error>
where
    T: cpal::Sample,
{
    let file = File::open("src/pater_emon.mp3")?;

    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;
    let envelope_len_ms_min = 1.0;
    let envelope_len_ms_max = 100.0;
    let envelope_len_samples_min = (sample_rate / (1000.0 / envelope_len_ms_min)) as usize;
    let envelope_len_samples_max = (sample_rate / (1000.0 / envelope_len_ms_max)) as usize;

    let mp3_source = Decoder::new(file).unwrap();
    let mp3_source_data: Vec<f32> = i16_array_to_f32(mp3_source.collect());

    let mut channel_1 = Vec::new();
    let mut channel_1_envelope = Vec::new();
    let mut channel_2 = Vec::new();
    let mut channel_2_envelope = Vec::new();

    // Called for every audio frame to generate appropriate sample
    let mut next_value = move || {
        let mut rng = rand::thread_rng();

        
        // if no more samples, add audio to the channel & a matching envolope
        if let None = channel_1.first() {
            let envolope_len_samples = rng.gen_range(envelope_len_samples_min..envelope_len_samples_max);
            let random_index = rng.gen_range(0..(mp3_source_data.len() - envolope_len_samples));
            channel_1.extend_from_slice(
                &mp3_source_data[random_index..(random_index + envolope_len_samples)],
            );
            channel_1_envelope = generate_envolope_of_len(channel_1.len())
        }
        if let None = channel_2.first() {
            let envolope_len_samples = rng.gen_range(envelope_len_samples_min..envelope_len_samples_max);
            let random_index = rng.gen_range(0..(mp3_source_data.len() - envolope_len_samples));
            channel_2.extend_from_slice(
                &mp3_source_data[random_index..(random_index + envolope_len_samples)],
            );
            channel_2_envelope = generate_envolope_of_len(channel_2.len())
        }

        let frame_1 = channel_1.remove(0) * channel_1_envelope.remove(0);
        let frame_2 = channel_2.remove(0) * channel_2_envelope.remove(0);

        (frame_1, frame_2)
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
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()),
    }
}
