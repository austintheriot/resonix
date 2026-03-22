use crate::{implementations::AudioBuffer, primitives::Sample};

use alloc::vec::Vec;

pub fn inputs_from_buffer_mapping<'a, 'b: 'a, Id: Into<usize> + Copy>(
    buffer_key_mapping: &'a [(Id, &'b [Sample], usize)],
) -> Vec<Option<AudioBuffer<'b>>> {
    let Some(max_id) = buffer_key_mapping
        .iter()
        .max_by_key(|(id, _, _)| <Id as Into<usize>>::into(*id))
        .map(|(id, ..)| <Id as Into<usize>>::into(*id))
    else {
        // no inputs, so no max, so no buffers to return
        return Vec::new();
    };

    let mut inputs: Vec<Option<AudioBuffer<'b>>> = (0..=max_id).map(|_| None).collect();

    buffer_key_mapping
        .iter()
        .for_each(|(id, raw_input_buffer, channels)| {
            let input_audio_buffer = AudioBuffer::new(raw_input_buffer, *channels)
                .expect("should be able to make audio_buffer out of raw buffer");

            inputs[<Id as Into<usize>>::into(*id)] = Some(input_audio_buffer);
        });

    inputs
}
