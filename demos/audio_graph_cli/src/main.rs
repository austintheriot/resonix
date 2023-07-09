use std::time::Duration;

use resonix::{AudioContext, DACNode, PassThroughNode, SineNode};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let mut audio_context = AudioContext::new();
    let sine_node = SineNode::new(2, 440.0);
    let sine_node_handle = audio_context.add_node(sine_node).unwrap();
    let pass_through_node = PassThroughNode::new(2);
    let pass_through_node_handle = audio_context.add_node(pass_through_node).unwrap();
    audio_context
        .connect(&sine_node_handle, &pass_through_node_handle)
        .unwrap();

    let mut prev_node_index = pass_through_node_handle;

    // string many pass-through nodes together to stress test audio
    for _ in 0..1000 {
        let pass_through_node = PassThroughNode::new(2);
        let pass_through_node_handle = audio_context.add_node(pass_through_node).unwrap();
        audio_context
            .connect(prev_node_index, &pass_through_node_handle)
            .unwrap();
        prev_node_index = pass_through_node_handle;
    }

    let dac_node = DACNode::new(2);
    let dac_node_index = audio_context.add_node(dac_node).unwrap();
    audio_context
        .connect(prev_node_index, dac_node_index)
        .unwrap();

    let mut audio_context = audio_context.into_audio_init().unwrap();
    audio_context.play_stream().unwrap();

    sine_node_handle.set_frequency_async(&mut audio_context, 440.0).await.unwrap();

    tokio::time::sleep(Duration::MAX).await;
}
