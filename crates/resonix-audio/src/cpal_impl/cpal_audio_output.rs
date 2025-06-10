use alloc::boxed::Box;
use cpal::{
    Sample, SizedSample, Stream, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use ringbuf::traits::Split;
use ringbuf::{HeapRb, traits::Observer};

use crate::{Consumer, CpalAudioOutputError, Producer, SystemAudioOutput, SystemAudioOutputError};

pub struct CpalAudioOutput<S: Sample> {
    producer: Producer<S>,
    // must be kept alive so stream doesn't end
    #[allow(dead_code)]
    stream: Stream,
    config: StreamConfig,
    ring_buffer_capacity: usize,
}

impl<S: Sample> CpalAudioOutput<S> {
    pub fn config(&self) -> &StreamConfig {
        &self.config
    }

    pub fn ring_buffer_capacity(&self) -> usize {
        self.ring_buffer_capacity
    }
}

impl<S: Sample + SizedSample + Send + 'static> CpalAudioOutput<S> {
    pub fn from_defaults() -> Self {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("failed to find a default output device");
        let supported_config = device.default_output_config().unwrap();
        let channels = supported_config.channels() as usize;

        // ~2.67ms of latency at 48kHz = ~128 samples--good enough for most needs
        let ring_buffer_capacity = supported_config.sample_rate().0 as f32 * 0.00267;
        let ring_buffer_capacity = ring_buffer_capacity.round() as usize;
        let buffer = HeapRb::new(ring_buffer_capacity);

        // setup ringbuffer to relay messages to the audio thread
        let (producer, consumer) = buffer.split();
        let mut consumer = Consumer::<S>(consumer);

        // just output whatever is read from the ring buffer
        let mut next_value = move || consumer.read().unwrap();
        let err_fn = |_err| unimplemented!();

        let stream = device
            .build_output_stream(
                &supported_config.config(),
                move |data: &mut [S], _| write_data(data, channels, &mut next_value),
                err_fn,
                None,
            )
            .unwrap();

        stream.play().unwrap();

        Self {
            stream,
            config: supported_config.config(),
            producer: Producer(producer),
            ring_buffer_capacity,
        }
    }
}

fn write_data<S>(output: &mut [S], channels: usize, next_sample: &mut dyn FnMut() -> S)
where
    S: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        for sample in frame.iter_mut() {
            *sample = next_sample();
        }
    }
}

impl<S> SystemAudioOutput<S> for CpalAudioOutput<S>
where
    S: Sample,
{
    fn write_sample(&mut self, sample: S) -> Result<(), SystemAudioOutputError> {
        self.producer.write(sample).map_err(|_sample| {
            SystemAudioOutputError::WriteError(Box::new(CpalAudioOutputError::WriteError))
        })?;

        Ok(())
    }

    #[cfg(feature = "mock")]
    fn consumer(&mut self) -> Option<Consumer<S>> {
        // only used in Mock implementation
        None
    }

    fn ready_for_sample(&self) -> bool {
        !self.producer.is_full()
    }
}
