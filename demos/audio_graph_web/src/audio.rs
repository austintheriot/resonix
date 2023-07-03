use resonix::{AudioContext, DACBuildError, DACNode, PassThroughNode, SineNode};

pub async fn set_up_audio_graph() -> Result<AudioContext, DACBuildError> {
    let mut audio_context = AudioContext::new();
    let sine_node = audio_context.new_sine_node(44100, 440.0);
    let sine_node_index = audio_context.add_node(sine_node).await.unwrap();

    let pass_through_node = audio_context.new_pass_through_node();
    let pass_through_node_handle = audio_context.add_node(pass_through_node).await.unwrap();
    audio_context
        .connect(&sine_node_index, &pass_through_node_handle)
        .await
        .unwrap();

    let mut prev_node_handle = pass_through_node_handle;

    // string many pass-through nodes together to stress test audio
    for _ in 0..250 {
        let pass_through_node = audio_context.new_pass_through_node();
        let pass_through_node_handle = audio_context.add_node(pass_through_node).await.unwrap();
        audio_context
            .connect(prev_node_handle, &pass_through_node_handle)
            .await
            .unwrap();
        prev_node_handle = pass_through_node_handle;
    }

    let dac_node = audio_context.new_dac_node();
    let dac_node_index = audio_context.add_node(dac_node).await.unwrap();
    audio_context
        .connect(prev_node_handle, dac_node_index)
        .await
        .unwrap();

    audio_context.initialize_dac_from_defaults().unwrap();
    audio_context.play_stream().unwrap();

    Ok(audio_context)
}
