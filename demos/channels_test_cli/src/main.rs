use std::time::Duration;

use log::info;
use resonix::{AudioContext, DACNode, SineNode};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let mut audio_context = AudioContext::new();
    audio_context.initialize_dac_from_defaults().unwrap();
    audio_context.play_stream().unwrap();

    let sine_node = audio_context.new_sine_node(audio_context.sample_rate().unwrap(), 440.0);
    let sine_node_index = audio_context.add_node(sine_node).await.unwrap();
    info!("main.rs sine_node_index = {sine_node_index:?}");

    let dac_node = audio_context.new_dac_node();
    let dac_node_index = audio_context.add_node(dac_node).await.unwrap();
    info!("main.rs dac_node_index = {dac_node_index:?}");

    let edge_index = audio_context
        .connect(sine_node_index, dac_node_index)
        .await
        .unwrap();
    info!("main.rs edge_index = {edge_index:?}");

    tokio::time::sleep(Duration::MAX).await;
}
