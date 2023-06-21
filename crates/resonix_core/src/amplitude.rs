use std::ops::Div;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Default)]
pub struct PeakAmplitude {
    pub peak_amplitude: f32,
    pub buffer_index: usize,
}

/// Returns `None` if buffer is empty
///
/// In the `PeakAmplitude` result, `peak_amplitude` is the absolute
/// value of the original sample (since Amplitude can never be negative)
///
///  Guaranteed to return a 0 or positive result if buffer is non-empty.
pub fn peak_amplitude(buffer: &[f32]) -> Option<PeakAmplitude> {
    buffer
        .iter()
        .enumerate()
        .filter(|(_, n)| !n.is_nan())
        .map(|(i, n)| (i, n.abs()))
        .max_by(|(_, a), (_, b)| {
            a.partial_cmp(b)
                .expect("Could not compare values in buffer to find peak amplitude")
        })
        .map(|(buffer_index, peak_amplitude)| PeakAmplitude {
            peak_amplitude,
            buffer_index,
        })
}

#[cfg(test)]
mod test_peak_amplitude {
    use crate::{peak_amplitude, PeakAmplitude};

    #[test]
    fn it_should_return_max_amplitude() {
        let buffer = vec![0.2, -1.0, 0.0, 0.5];
        let result = peak_amplitude(&buffer);

        assert_eq!(
            result,
            Some(PeakAmplitude {
                buffer_index: 1,
                peak_amplitude: 1.0
            })
        );
    }

    #[test]
    fn it_should_ignore_nans() {
        let buffer = vec![0.2, f32::NAN, -1.0, 0.0, 0.5];
        let result = peak_amplitude(&buffer);

        assert_eq!(
            result,
            Some(PeakAmplitude {
                buffer_index: 2,
                peak_amplitude: 1.0
            })
        );
    }

    #[test]
    fn it_should_return_none_if_buffer_is_empty() {
        let buffer = vec![];
        let result = peak_amplitude(&buffer);

        assert_eq!(result, None);
    }

    #[test]
    fn it_should_return_0_if_all_samples_are_0() {
        let buffer = vec![0.0; 100];
        let result = peak_amplitude(&buffer);

        assert_eq!(
            result,
            Some(PeakAmplitude {
                peak_amplitude: 0.0,
                buffer_index: 99
            })
        );
    }
}

/// Returns `None` if buffer is empty
///
/// Guaranteed to return a 0 or positive result if buffer is non-empty.
pub fn mean_square_root_amplitude(buffer: &[f32]) -> Option<f32> {
    if buffer.is_empty() {
        return None;
    }

    let filtered_buffer = buffer.iter().filter(|n| !n.is_nan()).collect::<Vec<&f32>>();

    Some(
        filtered_buffer
            .iter()
            .map(|n| n.powi(2))
            .sum::<f32>()
            .div(filtered_buffer.len() as f32)
            .sqrt(),
    )
}

#[cfg(test)]
mod test_mean_square_root_amplitude {
    use resonix_test_utils::assert_difference_is_within_tolerance;

    use crate::{mean_square_root_amplitude, Sine, SineInterface};

    #[test]
    pub fn it_should_return_correct_result_for_identical_samples() {
        let mut buffer = Vec::with_capacity(100);
        buffer.resize(100, 0.5);
        let result = mean_square_root_amplitude(&buffer);

        assert_eq!(result, Some(0.5));
    }

    #[test]
    pub fn it_should_return_correct_result_for_different_samples() {
        let mut sine = Sine::new();
        sine.set_sample_rate(44100).set_frequency(440.0);
        let mut buffer = vec![0.0; 44100];
        buffer.iter_mut().for_each(|n| *n = sine.next_sample());
        let result = mean_square_root_amplitude(&buffer).unwrap();

        // basic property of sinusoids that MSR ~ peak_amplitude / sqrt(2)
        assert_difference_is_within_tolerance(result, 1.0 / 2.0f32.sqrt(), 0.0000001);
    }

    #[test]
    fn it_should_ignore_nans() {
        let mut buffer = Vec::with_capacity(100);
        buffer.resize(100, 0.5);
        buffer[23] = f32::NAN;
        buffer[77] = f32::NAN;
        let result = mean_square_root_amplitude(&buffer);

        assert_eq!(result, Some(0.5));
    }

    #[test]
    fn it_should_return_none_if_buffer_is_empty() {
        let buffer = vec![];
        let result = mean_square_root_amplitude(&buffer);

        assert_eq!(result, None);
    }

    #[test]
    fn it_should_return_0_if_all_samples_are_0() {
        let buffer = vec![0.0; 100];
        let result = mean_square_root_amplitude(&buffer);

        assert_eq!(result, Some(0.0));
    }
}
