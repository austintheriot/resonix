#[cfg(all(test, feature = "mock"))]
mod test_audio_output {

    mod mock_audio_output {
        use resonix_audio::{
            implementations::{mock::MockAudioOutput, ringbuf::RingbufChannel},
            traits::SystemAudioOutput,
        };

        #[test]
        fn send_audio_sample() {
            let mut audio_output = MockAudioOutput::<f32>::new::<RingbufChannel>();
            let expected_output_sample: f32 = 0.123;

            audio_output
                .try_write_sample(expected_output_sample)
                .unwrap();

            let output_sample = audio_output.consumer().unwrap().try_read().unwrap();

            assert_eq!(output_sample, expected_output_sample)
        }

        #[test]
        fn send_audio_block() {
            let mut audio_output = MockAudioOutput::<f32>::new::<RingbufChannel>();
            let expected_output_block: [f32; 3] = [0.0, 1.0, 2.0];

            audio_output
                .try_write_block(&expected_output_block)
                .unwrap();

            let output_block = audio_output.consumer().unwrap().drain();
            assert_eq!(output_block.as_slice(), expected_output_block)
        }
    }

    mod mock_audio_input {
        use resonix_audio::{
            implementations::{mock::MockAudioInput, ringbuf::RingbufChannel},
            traits::SystemAudioInput,
        };

        #[test]
        fn receive_audio_sample() {
            let mut audio_input = MockAudioInput::<f32>::new::<RingbufChannel>();
            let expected_input_sample: f32 = 0.123;

            audio_input
                .producer()
                .unwrap()
                .try_write(expected_input_sample)
                .unwrap();

            let input_sample = audio_input.try_read_sample().unwrap();
            assert_eq!(input_sample, expected_input_sample)
        }

        #[test]
        fn receive_audio_block() {
            let mut audio_input = MockAudioInput::<f32>::new::<RingbufChannel>();
            let expected_input_block: [f32; 3] = [0.0, 1.0, 2.0];

            let mut producer = audio_input.producer().unwrap();
            for sample in expected_input_block {
                producer.try_write(sample).unwrap();
            }

            let input_block = audio_input.drain().unwrap();
            assert_eq!(input_block, expected_input_block)
        }
    }
}
