use std::time::Duration;

use resonix::{AudioContext, SineNode};

pub async fn set_up_audio_context() -> AudioContext {
    let mut audio_context = AudioContext::new();
    audio_context.initialize_dac_from_defaults().await.unwrap();
    audio_context.play_stream().unwrap();

    gloo_timers::future::sleep(Duration::from_secs(5)).await;

    let sine_node = SineNode::new_with_config(44100, 440.0);
    let _sine_node_index = audio_context.add_node(sine_node).await.unwrap();
    // info!("main.rs sine_node_index = {sine_node_index:?}");

    // let dac_node = DACNode::new();
    // let dac_node_index = audio_context.add_node(dac_node).await.unwrap();
    // info!("main.rs dac_node_index = {dac_node_index:?}");

    // audio_context.connect(sine_node_index, dac_node_index).await.unwrap();

    audio_context
}
