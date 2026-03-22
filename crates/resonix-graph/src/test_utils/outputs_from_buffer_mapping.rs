use crate::{implementations::AudioBufferMut, primitives::Sample};

use alloc::vec::Vec;

pub fn outputs_from_buffer_mapping<'a, 'b: 'a, Id: Into<usize> + Copy>(
    buffer_key_mapping: &'a mut [(Id, Option<&'b mut [Sample]>, usize)],
) -> Vec<Option<AudioBufferMut<'b>>> {
    let Some(max_id) = buffer_key_mapping
        .iter()
        .max_by_key(|(id, _, _)| <Id as Into<usize>>::into(*id))
        .map(|id| <Id as Into<usize>>::into(id.0))
    else {
        // no outputs, so no max, so no buffers to return
        return Vec::new();
    };

    let mut outputs: Vec<Option<AudioBufferMut<'b>>> = (0..=max_id).map(|_| None).collect();

    buffer_key_mapping
        .iter_mut()
        .for_each(|(id, raw_output_buffer, channels)| {
            if let Some(raw_output_buffer) = raw_output_buffer.take() {
                let output_audio_buffer = AudioBufferMut::new(raw_output_buffer, *channels)
                    .expect("should be able to make audio_buffer out of raw buffer");

                outputs[<Id as Into<usize>>::into(*id)] = Some(output_audio_buffer);
            }
        });

    outputs
}
