#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Default)]
pub struct PeakAmplitude {
    pub peak_amplitude: f32,
    pub buffer_index: usize,
}

/// Returns `None` if buffer is empty
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
        .map(|(buffer_index, _)| PeakAmplitude {
            peak_amplitude: buffer[buffer_index],
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
                peak_amplitude: -1.0
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
                peak_amplitude: -1.0
            })
        );
    }

    #[test]
    fn it_should_return_none_if_buffer_is_empty() {
        let buffer = vec![];
        let result = peak_amplitude(&buffer);

        assert_eq!(result, None);
    }
}
