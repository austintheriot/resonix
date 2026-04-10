use crate::{
    errors::AudioNodeRunError,
    primitives::{AudioNodeCtx, BlockSize, CurrentTime, PortId, SampleRate},
    test_utils::{
        InputBufferKeyMapping, OutputBufferKeyMapping, inputs_from_buffer_mapping,
        outputs_from_buffer_mapping,
    },
    traits::AudioNode,
};

/// Creates `inputs` and `outputs` buffer for an AudioNode, then runs
/// that node's `process` function
pub(crate) fn run_process<'mapping, 'buffer: 'mapping, N: AudioNode, Id: Into<usize> + Copy>(
    node: &mut N,
    input_buffer_key_mappings: Option<&'mapping [InputBufferKeyMapping<Id>]>,
    output_buffer_key_mappings: Option<&'mapping mut [OutputBufferKeyMapping<Id>]>,
    block_size: impl Into<BlockSize>,
    current_time: impl Into<CurrentTime>,
) -> Result<(), AudioNodeRunError> {
    let inputs = if let Some(input_buffer_key_mapping) = input_buffer_key_mappings {
        inputs_from_buffer_mapping(input_buffer_key_mapping)
    } else {
        inputs_from_buffer_mapping::<PortId>(&[])
    };
    let inputs = inputs.as_slice();

    let mut outputs = if let Some(output_buffer_key_mapping) = output_buffer_key_mappings {
        outputs_from_buffer_mapping(output_buffer_key_mapping)
    } else {
        outputs_from_buffer_mapping::<PortId>(&mut [])
    };
    let outputs = outputs.as_mut_slice();

    let ctx = AudioNodeCtx {
        block_size: block_size.into(),
        current_time: current_time.into(),
        sample_rate: SampleRate::default(),
    };

    node.process(inputs, outputs, ctx)?;

    Ok(())
}
