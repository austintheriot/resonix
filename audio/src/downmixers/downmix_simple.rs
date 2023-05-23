/// Mixes a multichannel frame down to a different number of output channels
///
/// Currently, should be the fastest implementation due to O(n) runtime and lack of sqrt() calculations
///
/// Creates a new buffer and writes into it. To avoid unnecessary allocations, use `downmix_simple_to_buffer`
pub fn downmix_simple(channels_in: &[f32], num_channels_out: u32) -> Vec<f32> {
    let mut output_buffer = vec![0.0; num_channels_out as usize];
    downmix_simple_to_buffer(channels_in, num_channels_out, &mut output_buffer);
    output_buffer
}

/// Mixes a multichannel frame down to a different number of output channels
///
/// Currently, should be the fastest implementation due to O(n) runtime and lack of sqrt() calculations
///
/// Writes into an existing buffer to avoid unnecessary allocations
pub fn downmix_simple_to_buffer<'a>(
    channels_in: &[f32],
    num_channels_out: u32,
    write_buffer: &'a mut Vec<f32>,
) -> &'a mut Vec<f32> {
    let channel_weight = 1.0 / num_channels_out as f32;

    let mut single_channel_output = 0.0;
    for sample in channels_in {
        single_channel_output += sample * channel_weight;
    }

    let normalization_factor = (num_channels_out as f32).sqrt().recip();
    single_channel_output *= normalization_factor;

    // copy output into buffer
    write_buffer.fill(single_channel_output);

    write_buffer
}
