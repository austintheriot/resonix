use resonix::{, AudioContext, DACBuildError, DACNode, PassThroughNode, SineNode, ProcessorInterface};

pub async fn set_up_audio_graph() -> Result<AudioContext, DACBuildError> {
    let mut audio_context = AudioContext::new();
    let sine_node_index = SineNode::new_with_config(44100, 440.0)
        .add_to_context(&mut audio_context)
        .unwrap();
    let pass_through_node_index = PassThroughNode::new()
        .add_to_context(&mut audio_context)
        .unwrap();
    audio_context
        .connect(sine_node_index, pass_through_node_index)
        .await
        .unwrap();

    let mut prev_node_index = pass_through_node_index;

    // string 500 pass-through nodes together to stress test audio
    for _ in 0..500 {
        let pass_through_node_index = PassThroughNode::new()
            .add_to_context(&mut audio_context)
            .unwrap();
        audio_context
            .connect(prev_node_index, pass_through_node_index)
            .await
            .unwrap();
        prev_node_index = pass_through_node_index;
    }

    let dac_node_index = DACNode::new().add_to_context(&mut audio_context).unwrap();
    audio_context
        .connect(prev_node_index, dac_node_index)
        .await
        .unwrap();

    audio_context.initialize_dac_from_defaults().await.unwrap();
    audio_context.play_stream().unwrap();

    Ok(audio_context)
}
