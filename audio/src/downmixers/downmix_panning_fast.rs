
/// Mixes a multichannel frame down to a different number of output channels
/// 
/// Faster, but less aurally correct implementation (does not call .sqrt()
/// on each run of the nested for loop)
pub fn downmix_panning_fast(channels_in: &[f32], num_channels_out: u32) -> Vec<f32> {
    if channels_in.is_empty() {
        return vec![0.0; num_channels_out as usize];
    }

    if num_channels_out == 0 {
        return Vec::new();
    }

    if channels_in.len() == num_channels_out as usize {
        return channels_in.to_vec();
    }

    let mut samples_out = vec![0.0; num_channels_out as usize];

    for (sample_in_i, sample_in) in channels_in.iter().enumerate() {
        for (sample_out_i, sample_out) in samples_out.iter_mut().enumerate() {
            let sample_in_index_progress = sample_in_i as f32 / channels_in.len() as f32;
            let sample_out_index_progress = sample_out_i as f32 / num_channels_out as f32;

            // how "far away" the input channel is to the output channel (as a percentage)
            let index_difference = (sample_out_index_progress - sample_in_index_progress).abs();
            // multiplier makes input channels that are "closest" to the ouput channel louder
            let amplitude_multiplier = 1.0 - index_difference;

            let value_to_add = sample_in * amplitude_multiplier;
            *sample_out += value_to_add;
        }
    }

    // hacky solution for now (@todo: research multi-channel mixdown algorithms):
    // the more voices that are getting "squished" into fewervoices,
    // the quieter the mixed down amplitude should be, and vice versa:
    // 2 voices scaled to 500 voices should be amplified
    // and 500 voices scaled to 2 should be de-amplified
    // in my own aural experimentation, the `cbrt` function seems to approximate this relationship well
    let scale_divisor = (channels_in.len() as f32 / num_channels_out as f32).cbrt();
    for sample in &mut samples_out {
        *sample = *sample / scale_divisor;
    }

    samples_out
}
