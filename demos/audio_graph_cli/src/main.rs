use std::time::Duration;

use resonix::{AudioContext, DACNode, PassThroughNode, SineNode};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let mut audio_context = AudioContext::new();
    let sine_node = SineNode::new_with_config(44100, 440.0);
    let sine_node_handle = audio_context.add_node(sine_node).await.unwrap();
    let pass_through_node = PassThroughNode::new();
    let pass_through_node_handle = audio_context.add_node(pass_through_node).await.unwrap();
    audio_context
        .connect(&sine_node_handle, &pass_through_node_handle)
        .await
        .unwrap();

    let mut prev_node_index = pass_through_node_handle;

    // string many pass-through nodes together to stress test audio
    for _ in 0..1000 {
        let pass_through_node = PassThroughNode::new();
        let pass_through_node_handle = audio_context.add_node(pass_through_node).await.unwrap();
        audio_context
            .connect(prev_node_index, &pass_through_node_handle)
            .await
            .unwrap();
        prev_node_index = pass_through_node_handle;
    }

    let dac_node = DACNode::new();
    let dac_node_index = audio_context.add_node(dac_node).await.unwrap();
    audio_context
        .connect(prev_node_index, dac_node_index)
        .await
        .unwrap();

    audio_context.initialize_dac_from_defaults().await.unwrap();
    audio_context.play_stream().unwrap();

    let sample_rate = audio_context.sample_rate().unwrap();
    sine_node_handle.set_frequency(440.0).await.unwrap();

    println!("{:?}", sample_rate);

    tokio::time::sleep(Duration::MAX).await;
}
