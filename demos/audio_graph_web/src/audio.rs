use resonix::{AudioContext, Connect, DACBuildError, DACNode, SineNode, PassThroughNode, Node};

pub async fn set_up_audio_graph() -> Result<AudioContext, DACBuildError> {
    let mut audio_context = AudioContext::new();
    audio_context.initialize_dac_from_defaults().await?;

    let sample_rate = audio_context.sample_rate().unwrap();
    let sine_node = SineNode::new_with_config(&mut audio_context, sample_rate, 440.0);

    let pass_through_node = PassThroughNode::new(&mut audio_context);
    sine_node.connect(&pass_through_node).unwrap();

    let mut prev_node = pass_through_node;

    for _ in 0..100 {
        let pass_through_node = PassThroughNode::new(&mut audio_context);
        prev_node.connect(&pass_through_node).unwrap();
        prev_node = pass_through_node;
    }

    let dac_node = DACNode::new(&mut audio_context);
    prev_node.connect(&dac_node).unwrap();

    Ok(audio_context)
}
