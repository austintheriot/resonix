#[cfg(all(test, feature = "mock"))]
mod test_audio_output {

    mod mock_audio_output {
        use cpal::Sample;
        use resonix_audio::{MockAudioOutput, SystemAudioOutput};
        use ringbuf::traits::consumer::Consumer;

        #[test]
        fn send_audio() {
            let mut audio_output = MockAudioOutput::<f32>::new();

            audio_output
                .write_sample(Sample::from_sample(0.123))
                .unwrap();

            let value = audio_output.consumer().unwrap().try_pop();
            assert_eq!(value, Some(0.123))
        }
    }

    mod mock_audio_input {
        use cpal::Sample;
        use resonix_audio::{MockAudioInput, SystemAudioInput};
        use ringbuf::traits::Producer;

        #[test]
        fn receive_audio() {
            let mut audio_input = MockAudioInput::<f32>::new();

            audio_input
                .producer()
                .unwrap()
                .try_push(Sample::from_sample(0.123))
                .unwrap();

            let value = audio_input.read_sample().unwrap();

            assert_eq!(value, 0.123)
        }
    }
}
