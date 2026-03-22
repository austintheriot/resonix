use crate::{
    implementations::AudioBufferMut,
    primitives::{PortId, Sample},
};

use alloc::vec::Vec;

pub fn outputs_from_buffer_mapping<'a, 'b: 'a>(
    buffer_key_mapping: &'a mut [(PortId, Option<&'b mut [Sample]>, usize)],
) -> Vec<Option<AudioBufferMut<'b>>> {
    let max_port_id = buffer_key_mapping
        .iter()
        .max_by_key(|(key, _, _)| key)
        .unwrap()
        .0;

    let mut outputs: Vec<Option<AudioBufferMut<'b>>> = (0..=**max_port_id).map(|_| None).collect();

    buffer_key_mapping
        .iter_mut()
        .for_each(|(port_id, raw_output_buffer, channels)| {
            if let Some(raw_output_buffer) = raw_output_buffer.take() {
                let output_audio_buffer = AudioBufferMut::new(raw_output_buffer, *channels)
                    .expect("should be able to make audio_buffer out of raw buffer");

                outputs[***port_id] = Some(output_audio_buffer);
            }
        });

    outputs
}
