use crate::{implementations::AudioBuffer, primitives::Sample};

use alloc::vec::Vec;

pub(crate) struct InputBufferKeyMapping<'buffer, Id: Into<usize> + Copy> {
    pub id: Id,
    pub buffer: &'buffer [Sample],
    pub num_channels: usize,
}

/// id, buffer, channels
pub(crate) fn inputs_from_buffer_mapping<'args, 'buffer: 'args, Id: Into<usize> + Copy>(
    buffer_key_mapping: &'args [InputBufferKeyMapping<'buffer, Id>],
) -> Vec<Option<AudioBuffer<'buffer>>> {
    let Some(max_id) = buffer_key_mapping
        .iter()
        .max_by_key(|buffer_key_mapping: &&InputBufferKeyMapping<Id>| {
            <Id as Into<usize>>::into(buffer_key_mapping.id)
        })
        .map(|mapping| <Id as Into<usize>>::into(mapping.id))
    else {
        // no outputs, so no max, so no buffers to return
        return Vec::new();
    };

    let mut inputs: Vec<Option<AudioBuffer<'buffer>>> = (0..=max_id).map(|_| None).collect();

    buffer_key_mapping.iter().for_each(
        |InputBufferKeyMapping {
             buffer,
             num_channels,
             id,
         }: &InputBufferKeyMapping<Id>| {
            let input_audio_buffer = AudioBuffer::new(buffer, *num_channels)
                .expect("should be able to make audio_buffer out of raw buffer");

            inputs[<Id as Into<usize>>::into(*id)] = Some(input_audio_buffer);
        },
    );

    inputs
}
