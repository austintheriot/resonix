use std::time::Duration;

use resonix::{AudioContext, Connect, DACNode, PassThroughNode, SineInterface, SineNode};

#[tokio::main]
async fn main() {
    let mut audio_context = AudioContext::new();
    let mut sine_node = SineNode::new(&mut audio_context);

    let pass_through_node = PassThroughNode::new(&mut audio_context);
    sine_node.connect(&pass_through_node).unwrap();

    let mut prev_node = pass_through_node;

    // string 50 pass-through nodes together to stress test audio
    for _ in 0..50 {
        let pass_through_node = PassThroughNode::new(&mut audio_context);
        prev_node.connect(&pass_through_node).unwrap();
        prev_node = pass_through_node;
    }

    let dac_node = DACNode::new(&mut audio_context);
    prev_node.connect(&dac_node).unwrap();

    audio_context.initialize_dac_from_defaults().await.unwrap();
    audio_context.play_stream().unwrap();

    let sample_rate = audio_context.sample_rate().unwrap();
    sine_node.set_sample_rate(sample_rate).set_frequency(440.0);

    tokio::time::sleep(Duration::MAX).await;
}
