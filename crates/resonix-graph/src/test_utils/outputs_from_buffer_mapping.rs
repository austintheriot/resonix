use crate::{implementations::AudioBufferMut, primitives::Sample};

use alloc::vec::Vec;

pub(crate) struct OutputBufferKeyMapping<'buffer, Id: Into<usize> + Copy> {
    pub id: Id,
    pub buffer: Option<&'buffer mut [Sample]>,
    pub num_channels: usize,
}

pub(crate) fn outputs_from_buffer_mapping<'mapping, 'buffer: 'mapping, Id: Into<usize> + Copy>(
    buffer_key_mapping: &'buffer mut [OutputBufferKeyMapping<Id>],
) -> Vec<Option<AudioBufferMut<'buffer>>> {
    let Some(max_id) = buffer_key_mapping
        .iter()
        .max_by_key(|buffer_key_mapping: &&OutputBufferKeyMapping<Id>| {
            <Id as Into<usize>>::into(buffer_key_mapping.id)
        })
        .map(|mapping| <Id as Into<usize>>::into(mapping.id))
    else {
        // no outputs, so no max, so no buffers to return
        return Vec::new();
    };

    let mut outputs: Vec<Option<AudioBufferMut<'buffer>>> = (0..=max_id).map(|_| None).collect();

    buffer_key_mapping
        .iter_mut()
        .for_each(|mapping: &mut OutputBufferKeyMapping<Id>| {
            if let Some(raw_output_buffer) = mapping.buffer.take() {
                let output_audio_buffer =
                    AudioBufferMut::new(raw_output_buffer, mapping.num_channels)
                        .expect("should be able to make audio_buffer out of raw buffer");

                outputs[<Id as Into<usize>>::into(mapping.id)] = Some(output_audio_buffer);
            }
        });

    outputs
}
