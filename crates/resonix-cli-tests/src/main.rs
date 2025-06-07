#[cfg(not(test))]
use std::{thread::sleep, time::Duration};

#[cfg(not(test))]
use resonix_audio::SystemAudioOutput;

#[cfg(not(test))]
type AudioOutput<S> = resonix_audio::CpalAudioOutput<S>;

#[cfg(test)]
type AudioOutput<S> = resonix_audio::MockAudioOutput<S>;

#[cfg(not(test))]
fn main() {
    use core::f32;

    let mut audio_output = AudioOutput::from_defaults();

    let mut sample_clock = 0f32;
    let sample_rate = audio_output.config().sample_rate.0;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate as f32;
        let octave = 1.0;
        (sample_clock * 440.0 * octave * f32::consts::PI / sample_rate as f32).sin()
    };

    for _ in 0..5 {
        for _ in 0..sample_rate {
            audio_output.write_sample(next_value()).unwrap();
        }
        sleep(Duration::from_millis(500));
    }
}

#[cfg(test)]
mod test_mock {
    use dasp_sample::Sample;
    use resonix_audio::SystemAudioOutput;

    use crate::AudioOutput;

    #[test]
    fn output_is_captured() {
        let mut audio_output = AudioOutput::<f32>::new();

        audio_output
            .write_sample(Sample::from_sample(0.123))
            .unwrap();

        let value = audio_output.consumer().unwrap().read().unwrap();

        assert_eq!(value, 0.123)
    }
}
