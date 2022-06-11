extern crate anyhow;
extern crate clap;
extern crate cpal;

use audio::clock::Clock;
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
fn write_data<T>(output_data: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: cpal::Sample,
{
    for frame in output_data.chunks_mut(channels) {
        let value: T = cpal::Sample::from::<f32>(&next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

fn generate_sine(sample_tick: f64, frequency: f64, frequency_modulation: f64) -> f64 {
    (2.0 * PI * (frequency * sample_tick + frequency_modulation)).sin()
}

/// Generates only the top portion of a sine wave (good envelope for granular synthesis)
///
/// The sine index is assumed to range from 0.0 -> 1.0
fn generate_sine_envelope(current_index: f32) -> f32 {
    (current_index * std::f32::consts::PI).sin()
}

/// Shuffles mp3 data in "chunks" -- good for a "faked" granular synthesis
fn shuffle_mp3_data<T: Clone>(data: &Vec<T>, shuffle_size: usize) -> Vec<T> {
    let mut rng = rand::thread_rng();

    let mut chunked_data: Vec<&[T]> = data.chunks(shuffle_size).collect();
    chunked_data.shuffle(&mut rng);
    let mut shuffled_data = Vec::with_capacity(data.len());
    for a in chunked_data.iter_mut() {
        shuffled_data.extend_from_slice(a);
    }

    shuffled_data
}

fn i16_array_to_f32(data: Vec<i16>) -> Vec<f32> {
    data.into_iter().map(|el| el as f32 / 65536.0).collect()
}

fn generate_envolopes(len: usize, envolope_samples_len: usize) -> Vec<f32> {
    let mut envelope_data = Vec::with_capacity(len);
    for (i, _) in (0..len).enumerate() {
        let i = i % envolope_samples_len;
        let percentage = i as f32 / envolope_samples_len as f32;
        envelope_data.push(generate_sine_envelope(percentage));
    }
    envelope_data
}

pub fn run<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    clock: Clock,
) -> Result<(), anyhow::Error>
where
    T: cpal::Sample,
{
    let file = File::open("src/pater_emon.mp3")?;

    let runtime_seconds = 30.0;
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;
    let envolope_samples_len = (sample_rate / 10.0) as usize;

    let mp3_source = Decoder::new(file).unwrap();
    let mp3_source_data: Vec<f32> = i16_array_to_f32(mp3_source.collect());
    let mut mp3_data_1 = shuffle_mp3_data(&mp3_source_data, envolope_samples_len).into_iter();
    let mut mp3_data_2 = shuffle_mp3_data(&mp3_source_data, envolope_samples_len).into_iter();
    let mut envelope_data = generate_envolopes(mp3_data_1.len(), envolope_samples_len).into_iter();

    // Called for every audio frame to generate appropriate sample
    let mut next_value = move || {
        let frame_1 = mp3_data_1.next();
        let frame_2 = mp3_data_2.next();
        let envolope_volume = envelope_data.next();

        return if let (Some(frame_1),Some(frame_2), Some(envelope_gain)) = (frame_1, frame_2, envolope_volume) {
            let frame_1 = frame_1 * 0.5;
            let frame_2 = frame_2 * 0.5;
            ((frame_1 + frame_2) * envelope_gain).clamp(-0.75, 0.75)
        } else {
            0.0
        };
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

    std::thread::sleep(std::time::Duration::from_secs_f32(runtime_seconds));

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

    let clock = Clock::default();

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), clock),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), clock),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), clock),
    }
}
