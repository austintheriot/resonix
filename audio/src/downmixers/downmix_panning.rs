/// Mixes a multichannel frame down to a different number of output channels
/// 
/// Slower, more aurally "correct" implementation that preservers overall energy / amplitude.
pub fn downmix_panning(channels_in: &[f32], num_channels_out: u32) -> Vec<f32> {
    if channels_in.is_empty() {
        return vec![0.0; num_channels_out as usize];
    }

    if num_channels_out == 0 {
        return Vec::new();
    }

    if channels_in.len() == num_channels_out as usize {
        return channels_in.to_vec();
    }

    let num_channels_in = channels_in.len();
    let mut samples_out = vec![0.0; num_channels_out as usize];

    for (sample_in_i, sample_in) in channels_in.iter().enumerate() {
        for (sample_out_i, sample_out) in samples_out.iter_mut().enumerate() {
            let sample_in_index_progress = sample_in_i as f32 / (num_channels_in - 1) as f32;
            let sample_out_index_progress = sample_out_i as f32 / (num_channels_out - 1) as f32;

            // maintains left-to-right panning while downmixing
            // note: using .sqrt() here may help to maintain the perceived
            // loudness and energy of the audio while panning across channels
            let amplitude_multiplier =
                (1.0 - (sample_in_index_progress - sample_out_index_progress).abs()).sqrt();
            *sample_out += sample_in * amplitude_multiplier;
        }
    }

    let normalization_factor = (num_channels_out as f32).sqrt();
    samples_out
        .iter_mut()
        .for_each(|sample| *sample /= normalization_factor);

    samples_out
}