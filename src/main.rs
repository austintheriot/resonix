extern crate anyhow;
extern crate clap;
extern crate cpal;

use std::{f64::consts::PI, sync::{Arc, Mutex}};
use clap::arg;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

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

pub fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig, clock: Clock) -> Result<(), anyhow::Error>
where
    T: cpal::Sample,
{
    let seconds = 30.0;
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;
    let gain = 0.2;

    // Called for every audio frame to generate appropriate sample
    let mut next_value = move || {
        let now = Arc::clone(&clock.now);
        let mut now = now.lock().unwrap();
        *now += 1;

        let sample_tick = (*now as f64) / (sample_rate as f64);
       
        let modulation_speed_hz = 0.25;
        let modulation_depth = 220.0;
        let modulation_frequency = generate_sine(sample_tick, modulation_speed_hz, 0.0) * modulation_depth;
       
        // modulate the frequency of a sine wave
        let sample = generate_sine(sample_tick, 440.0, modulation_frequency);

        // drop volume
        let sample = sample * gain;

        // escape hatch--clip amplitude
        sample.clamp(-0.75, 0.75) as f32
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

    std::thread::sleep(std::time::Duration::from_secs_f32(seconds));

    Ok(())
}

#[derive(Default)]
pub struct Clock {
    /// a u32 that is incremented 48,000 times a second will overflow after a day
    pub now: Arc<Mutex<u32>>,
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
