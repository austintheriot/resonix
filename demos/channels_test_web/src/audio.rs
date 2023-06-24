use std::sync::Arc;

use log::info;
use resonix::{cpal, DACConfig, Sine, SineInterface, DAC, AudioContext, SineNode, ProcessorInterface, DACNode};

pub async fn set_up_audio_context() -> AudioContext {
    let mut audio_context = AudioContext::new();
    audio_context.initialize_dac_from_defaults().await.unwrap();
    audio_context.play_stream().unwrap();
    let sine_node = SineNode::new_with_config(44100, 440.0);
    let sine_node_index = audio_context.add_node(sine_node).await.unwrap();
    // info!("main.rs sine_node_index = {sine_node_index:?}");

    // let dac_node = DACNode::new();
    // let dac_node_index = audio_context.add_node(dac_node).await.unwrap();
    // info!("main.rs dac_node_index = {dac_node_index:?}");

    // audio_context.connect(sine_node_index, dac_node_index).await.unwrap();

    audio_context
}