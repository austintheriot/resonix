#[cfg(all(test, feature = "test"))]
mod test_audio_output {
    use cpal::Sample;
    use resonix_audio::{SystemAudioOutput, TestAudioOutput};
    use ringbuf::traits::consumer::Consumer;

    #[test]
    fn test_receives_audio() {
        let (mut audio_output, mut consumer) = TestAudioOutput::<f32>::new();

        audio_output
            .write_sample(Sample::from_sample(0.123))
            .unwrap();

        let value = consumer.try_pop();
        assert_eq!(value, Some(0.123))
    }
}
