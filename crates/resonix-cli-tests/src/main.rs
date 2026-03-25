use resonix_audio::{
    implementations::{
        cpal::CpalAudioOutput,
        ringbuf::{self, RingbufConsumer, RingbufProducer, ringbuf::traits::Split},
    },
    traits::SystemAudioOutput,
};

fn main() {
    use core::f32;

    // TODO: implement this logic in the library
    // so callers don't have to deal with this logic
    let channels = ringbuf::ringbuf::HeapRb::new(2048);
    let (producer, consumer) = channels.split();
    let producer = RingbufProducer::new(producer);
    let consumer = RingbufConsumer::new(consumer);
    let mut audio_output = CpalAudioOutput::from_defaults(consumer, producer);

    let mut sample_clock = 0f32;
    let sample_rate = audio_output.config().sample_rate.0;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate as f32;
        let octave = 1.0;
        (sample_clock * 440.0 * octave * f32::consts::PI / sample_rate as f32).sin()
    };

    // just fill the buffer on every loop
    loop {
        while audio_output.ready_for_sample() {
            audio_output.try_write_sample(next_value()).unwrap();
        }
    }
}

#[cfg(test)]
mod test_mock {
    use dasp_sample::Sample;
    use resonix_audio::{MockAudioOutput, SystemAudioOutput};

    #[test]
    fn output_is_captured() {
        let mut audio_output = MockAudioOutput::<f32>::default();

        audio_output
            .try_write_sample(Sample::from_sample(0.123))
            .unwrap();

        let value = audio_output.consumer().unwrap().try_read().unwrap();

        assert_eq!(value, 0.123)
    }
}
