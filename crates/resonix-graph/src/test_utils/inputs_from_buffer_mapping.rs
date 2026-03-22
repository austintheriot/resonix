use crate::{
    implementations::AudioBuffer,
    primitives::{PortId, Sample},
};

use alloc::vec::Vec;

pub fn inputs_from_buffer_mapping<'a, 'b: 'a>(
    buffer_key_mapping: &'a [(PortId, &'b [Sample], usize)],
) -> Vec<Option<AudioBuffer<'b>>> {
    // no inputs, so no max, so no buffers to return
    let Some((max_port_id, ..)) = buffer_key_mapping.iter().max_by_key(|(key, _, _)| key) else {
        return Vec::new();
    };

    let mut inputs: Vec<Option<AudioBuffer<'b>>> = (0..=***max_port_id).map(|_| None).collect();

    buffer_key_mapping
        .iter()
        .for_each(|(port_id, raw_input_buffer, channels)| {
            let input_audio_buffer = AudioBuffer::new(raw_input_buffer, *channels)
                .expect("should be able to make audio_buffer out of raw buffer");

            inputs[***port_id] = Some(input_audio_buffer);
        });

    inputs
}
