use resonix::{AudioContext, Connect, DACBuildError, DACNode, SineNode};

pub async fn set_up_audio_graph() -> Result<AudioContext, DACBuildError> {
    let mut audio_context = AudioContext::new();
    audio_context.initialize_dac_from_defaults().await?;

    let sample_rate = audio_context.sample_rate().unwrap();
    let sine_node = SineNode::new_with_config(&mut audio_context, sample_rate, 440.0);
    let dac_node = DACNode::new(&mut audio_context);
    sine_node.connect(&dac_node).unwrap();

    Ok(audio_context)
}
