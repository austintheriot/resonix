use core::marker::PhantomData;

use alloc::boxed::Box;
use cpal::{
    Sample, SizedSample, Stream, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use crate::{
    ProducerError, SystemAudioOutputError,
    traits::{Consumer, Producer, SystemAudioOutput},
};

pub struct CpalAudioOutput<S, P: Producer<S>> {
    producer: P,
    // must be kept alive so stream doesn't end
    #[allow(dead_code)]
    stream: Stream,
    config: StreamConfig,
    _phantom: PhantomData<S>,
}

impl<S: Sample, P: Producer<S>> CpalAudioOutput<S, P> {
    pub fn config(&self) -> &StreamConfig {
        &self.config
    }
}

impl<S: Sample + SizedSample + Send + 'static, P: Producer<S>> CpalAudioOutput<S, P> {
    /// requires a consumer & producer pair to propagate audio data from the audio thread
    pub fn from_defaults<C: Consumer<S> + Send + 'static>(mut consumer: C, producer: P) -> Self {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("failed to find a default output device");
        let supported_config = device.default_output_config().unwrap();
        let channels = supported_config.channels() as usize;

        // just output whatever is read from the ring buffer
        let mut next_value = move || consumer.try_read().unwrap();
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
            producer,
            _phantom: PhantomData,
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

impl<S: Copy, P: Producer<S>> SystemAudioOutput<S> for CpalAudioOutput<S, P>
where
    S: Sample,
{
    fn try_write_block(&mut self, samples: &[S]) -> Result<(), SystemAudioOutputError> {
        for sample in samples {
            self.producer.try_write(*sample)?;
        }

        Ok(())
    }

    fn try_write_sample(&mut self, sample: S) -> Result<(), SystemAudioOutputError> {
        self.producer.try_write(sample).map_err(|_e| {
            SystemAudioOutputError::ProducerError(ProducerError::InsufficientSpace)
        })?;

        Ok(())
    }

    fn ready_for_sample(&self) -> bool {
        self.producer.ready()
    }

    #[cfg(feature = "mock")]
    fn consumer(&mut self) -> Option<Box<dyn Consumer<S>>> {
        // only used in Mock implementation--we need the consumer
        // to send audio data to cpal
        None
    }
}
