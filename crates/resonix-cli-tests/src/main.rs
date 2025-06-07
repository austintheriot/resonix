use std::{thread::sleep, time::Duration};

use resonix_audio::{CpalAudioOutput, SystemAudioOutput};

fn main() {
    use core::f32;

    let mut audio_output = CpalAudioOutput::from_defaults();

    let mut sample_clock = 0f32;
    let sample_rate = audio_output.config().sample_rate.0;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate as f32;
        let octave = 1.0;
        (sample_clock * 440.0 * octave * f32::consts::PI / sample_rate as f32).sin()
    };

    // just fill the buffer on every loop
    loop {
        let audio_frame_time = Duration::from_millis(16);
        while audio_output.write_sample(next_value()).is_ok() {}
        sleep(audio_frame_time);
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
            .write_sample(Sample::from_sample(0.123))
            .unwrap();

        let value = audio_output.consumer().unwrap().read().unwrap();

        assert_eq!(value, 0.123)
    }
}
