#[cfg(all(test, feature = "mock"))]
mod test_audio_output {

    mod mock_audio_output {
        use cpal::Sample;
        use resonix_audio::{MockAudioOutput, SystemAudioOutput};

        #[test]
        fn send_audio() {
            let mut audio_output = MockAudioOutput::<f32>::new();

            audio_output
                .write_sample(Sample::from_sample(0.123))
                .unwrap();

            let value = audio_output.consumer().unwrap().read().unwrap();

            assert_eq!(value, 0.123)
        }
    }

    mod mock_audio_input {
        use cpal::Sample;
        use resonix_audio::{MockAudioInput, SystemAudioInput};

        #[test]
        fn receive_audio() {
            let mut audio_input = MockAudioInput::<f32>::new();

            audio_input
                .producer()
                .unwrap()
                .write(Sample::from_sample(0.123))
                .unwrap();

            let value = audio_input.read_sample().unwrap();

            assert_eq!(value, 0.123)
        }
    }
}
