/// Mixes a multichannel frame down to a different number of output channels
///
/// Slower, more aurally "correct" implementation that preservers overall energy / amplitude.
///
/// Creates a new buffer and writes into it. To avoid unnecessary allocations, use `downmix_panning_to_buffer`
pub fn downmix_panning(channels_in: &[f32], num_channels_out: u32) -> Vec<f32> {
    let mut output_buffer = vec![0.0; num_channels_out as usize];
    downmix_panning_to_buffer(channels_in, num_channels_out, &mut output_buffer);
    output_buffer
}

/// Mixes a multichannel frame down to a different number of output channels
///
/// Slower, more aurally "correct" implementation that preservers overall energy / amplitude.
///
/// Writes into an existing buffer to avoid unnecessary allocations
pub fn downmix_panning_to_buffer<'a>(
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

    let num_channels_in = channels_in.len();

    for (sample_in_i, sample_in) in channels_in.iter().enumerate() {
        for (sample_out_i, sample_out) in write_buffer.iter_mut().enumerate() {
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

    let normalization_factor = (num_channels_out as f32).powf(2.0);
    write_buffer
        .iter_mut()
        .for_each(|sample| *sample /= normalization_factor);

    write_buffer
}
