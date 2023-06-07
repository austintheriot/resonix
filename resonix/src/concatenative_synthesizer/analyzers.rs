use std::sync::Arc;

use rustfft::{num_complex::Complex, num_traits::Zero, FftPlanner};

pub fn analyze_pitches(buffer: Arc<Vec<f32>>, breakpoints: &[usize]) -> Vec<f32> {
    todo!()
}

fn calculate_average_frequency(buffer: &[f32], sample_rate: f32) -> Vec<Complex<f32>> {
    let fft_size = buffer.len();
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(buffer.len());

    // Convert the audio buffer to the complex number format
    let mut input: Vec<Complex<f32>> = buffer.iter().map(|&x| Complex::new(x, 0.0)).collect();

    fft.process(&mut input);

    input
}

#[cfg(test)]
mod test_analyzers {
    mod frequency_analysis {

        use std::{f32::consts::PI, vec};

        use crate::concatenative_synthesizer::analyzers::calculate_average_frequency;

        fn fill_buffer_with_sine_wave(buffer: &mut [f32], frequency: f32, sample_rate: usize) {
            let angular_frequency = 2.0 * PI * frequency / sample_rate as f32;
            let mut phase: f32 = 0.0;

            for sample in buffer.iter_mut() {
                *sample = phase.sin();
                phase += angular_frequency;

                if phase >= 2.0 * PI {
                    phase -= 2.0 * PI;
                }
            }
        }

        #[test]
        fn finds_loudest_frequency() {
            let sample_rate = 44100;
            let mut buffer = vec![0.0; 1024];
            fill_buffer_with_sine_wave(&mut buffer, 440.0, sample_rate);
            let result = calculate_average_frequency(&buffer, sample_rate as f32);
            let index_of_loudest_frequency = result.iter().enumerate().max_by(|a, b| {
                a.norm().partial_cmp(b)
            }).unwrap();
            let loudest_frequency =
                (index_of_loudest_frequency as f32 / buffer.len() as f32) * sample_rate as f32;

            assert_eq!(loudest_frequency, 440.0);
        }
    }
}
