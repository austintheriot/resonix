extern crate anyhow;
extern crate clap;
extern crate cpal;

use clap::arg;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rand::prelude::*;
use rodio::Decoder;
use std::{env, f64::consts::PI, fs::File};

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
fn generate_sine_envelope_value_from_percent(current_index: f32) -> f32 {
    (current_index * std::f32::consts::PI).sin()
}

/// The sine index (`current_index`) is assumed to range from 0.0 -> 1.0
fn generate_triangle_envelope_value_from_percent(current_index: f32) -> f32 {
    (((current_index - 0.5).abs() * -1.0) + 0.5) * 2.0
}

fn i16_array_to_f32(data: Vec<i16>) -> Vec<f32> {
    data.into_iter().map(|el| el as f32 / 65536.0).collect()
}


pub struct Grain {
    pub start_frame: usize,
    pub end_frame: usize,
    pub current_frame: usize,
    pub finished: bool,
    pub len: usize,
}

impl Default for Grain {
    fn default() -> Self {
        Self {
            start_frame: 0,
            current_frame: 0,
            end_frame: 0,
            finished: true,
            len: 0,
        }
    }
}

impl Grain {
    pub fn new(start_frame: usize, end_frame: usize) -> Self {
        debug_assert!(start_frame < end_frame);
        Grain {
            start_frame,
            current_frame: start_frame,
            end_frame,
            finished: false,
            len: end_frame - start_frame,
        }
    }
    pub fn get_next_frame(&mut self) -> Option<usize> {
        if self.finished {
            return None;
        }

        self.current_frame += 1;
        if self.current_frame == self.end_frame {
            self.finished = true;
        }

        Some(self.current_frame)
    }
}

pub fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), anyhow::Error>
where
    T: cpal::Sample,
{
    let file = File::open("src/pater_emon.mp3")?;

    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;
    let envelope_len_ms_min = 100.0;
    let envelope_len_ms_max = 1000.0;
    let envelope_len_samples_min = (sample_rate / (1000.0 / envelope_len_ms_min)) as usize;
    let envelope_len_samples_max = (sample_rate / (1000.0 / envelope_len_ms_max)) as usize;

    let mp3_source = Decoder::new(file).unwrap();
    let mp3_source_data: Vec<f32> = i16_array_to_f32(mp3_source.collect());

    const NUM_CHANNELS: usize = 500;

    let mut channels_grains: Vec<Grain> = Vec::with_capacity(NUM_CHANNELS);
    for _ in 0..NUM_CHANNELS {
        channels_grains.push(Grain::default());
    }

    // Called for every audio frame to generate appropriate sample
    let mut next_value = move || {
        let mut rng = rand::thread_rng();

        // grain length should not exceed max mp3 source data length
        debug_assert!(mp3_source_data.len() > envelope_len_samples_max);

        for grain in channels_grains.iter_mut() {
            // if no more samples, add audio to the channel & a matching grain envolope
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

        let frame_samples = channels_grains
            .iter_mut()
            .map(|grain| {
                debug_assert_eq!(grain.finished, false);
                debug_assert_eq!(grain.finished, false);
                let envelope_percent =
                    ((grain.current_frame - grain.start_frame) as f32) / (grain.len as f32);
                debug_assert!(envelope_percent >= 0.0, "{}", envelope_percent);
                debug_assert!(envelope_percent < 1.0, "{}", envelope_percent);
                let envelope_value = generate_triangle_envelope_value_from_percent(envelope_percent);
                let frame_index = grain
                    .get_next_frame()
                    .expect("Grain should have another frame available");
                (mp3_source_data[frame_index], envelope_value)
            })
            .collect::<Vec<(f32, f32)>>();

        let mut left = 0.0;
        for (i, (sample, envelope)) in frame_samples.iter().enumerate() {
            let spatialization_percent = 1.0 - (i as f32) / (frame_samples.len() as f32);
            let value_to_add = (sample * envelope * spatialization_percent) / (NUM_CHANNELS as f32);
            left += value_to_add;
        }

        let mut right = 0.0;
        for (i, (sample, envelope)) in frame_samples.iter().enumerate() {
            let spatialization_percent = (i as f32) / (frame_samples.len() as f32);
            let value_to_add = (sample * envelope * spatialization_percent) / (NUM_CHANNELS as f32);
            right += value_to_add;
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
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()),
    }
}
