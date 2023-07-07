use log::info;

use resonix::{AudioContext, DACNode, SineNode, AudioInit};

pub async fn set_up_audio_context() -> AudioContext<AudioInit> {
    let mut audio_context = AudioContext::new().into_audio_init().unwrap();
    audio_context.play_stream().unwrap();

    let sine_node = SineNode::new_with_config(2, audio_context.sample_rate().unwrap(), 440.0);
    let sine_node_index = audio_context.add_node(sine_node).await.unwrap();
    info!("main.rs sine_node_index = {sine_node_index:?}");

    let dac_node = DACNode::new(2);
    let dac_node_index = audio_context.add_node(dac_node).await.unwrap();
    info!("main.rs dac_node_index = {dac_node_index:?}");

    let edge_index = audio_context
        .connect(sine_node_index, dac_node_index)
        .await
        .unwrap();
    info!("main.rs edge_index = {edge_index:?}");

    audio_context
}
