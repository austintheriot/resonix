/// Mixes a multichannel frame down to a different number of output channels
///
/// Faster, but less aurally correct implementation (does not call .sqrt()
/// on each run of the nested for loop)
/// 
/// Creates a new buffer and writes into it. To avoid unnecessary allocations, use `downmix_panning_fast_to_buffer`
pub fn downmix_panning_fast(channels_in: &[f32], num_channels_out: u32) -> Vec<f32> {
    let mut output_buffer = vec![0.0; num_channels_out as usize];
    downmix_panning_fast_to_buffer(channels_in, num_channels_out, &mut output_buffer);
    output_buffer
}

/// Mixes a multichannel frame down to a different number of output channels
///
/// Faster, but less aurally correct implementation (does not call .sqrt()
/// on each run of the nested for loop)
/// 
/// Writes into an existing buffer to avoid unnecessary allocations
pub fn downmix_panning_fast_to_buffer<'a>(
    channels_in: &[f32],
    num_channels_out: u32,
    write_buffer: &'a mut [f32],
) -> &'a mut [f32] {
    if channels_in.is_empty() {
        return write_buffer;
    }

    if num_channels_out == 0 {
        return write_buffer;
    }

    if channels_in.len() == num_channels_out as usize {
        write_buffer.copy_from_slice(channels_in);
        return write_buffer;
    }

    for (sample_in_i, sample_in) in channels_in.iter().enumerate() {
        for (sample_out_i, sample_out) in write_buffer.iter_mut().enumerate() {
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
    for sample in write_buffer.iter_mut() {
        *sample /= scale_divisor;
    }

    write_buffer
}
