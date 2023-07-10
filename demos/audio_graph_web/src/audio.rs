use resonix::{AudioContext, AudioInit, DACBuildError, DACNode, PassThroughNode, SineNode};

pub async fn set_up_audio_graph() -> Result<AudioContext<AudioInit>, DACBuildError> {
    let mut audio_context = AudioContext::new();
    let sine_node = SineNode::new(1, 440.0);
    let sine_node_handle = audio_context.add_node(sine_node).unwrap();

    let pass_through_node = PassThroughNode::new(1);
    let pass_through_node_handle = audio_context.add_node(pass_through_node).unwrap();

    audio_context
        .connect(sine_node_handle, pass_through_node_handle)
        .unwrap();

    let mut prev_node_handle = pass_through_node_handle;

    // string many pass-through nodes together to stress test audio
    for _ in 0..150 {
        let pass_through_node = PassThroughNode::new(1);
        let pass_through_node_handle = audio_context.add_node(pass_through_node).unwrap();
        audio_context
            .connect(prev_node_handle, pass_through_node_handle)
            .unwrap();
        prev_node_handle = pass_through_node_handle;
    }

    let dac_node = DACNode::new(1);
    let dac_node_index = audio_context.add_node(dac_node).unwrap();
    audio_context
        .connect(prev_node_handle, dac_node_index)
        .unwrap();

    let mut audio_context = audio_context.into_audio_init().unwrap();
    audio_context.play_stream().unwrap();

    Ok(audio_context)
}
