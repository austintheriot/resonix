use std::time::Duration;

use resonix::{AddToContext, AudioContext, DACNode, PassThroughNode, SineNode, ProcessorInterface};

#[tokio::main]
async fn main() {
    let mut audio_context = AudioContext::new();
    let sine_node_index = SineNode::new_with_config(44100, 440.0)
        .add_to_context(&mut audio_context)
        .await
        .unwrap();
    let pass_through_node_index = PassThroughNode::new()
        .add_to_context(&mut audio_context)
        .await
        .unwrap();
    audio_context
        .connect(sine_node_index, pass_through_node_index).await
        .unwrap();

    let mut prev_node_index = pass_through_node_index;

    // string many pass-through nodes together to stress test audio
    for _ in 0..3000 {
        let pass_through_node_index = PassThroughNode::new()
            .add_to_context(&mut audio_context)
            .await
            .unwrap();
        audio_context
            .connect(prev_node_index, pass_through_node_index)
            .await
            .unwrap();
        prev_node_index = pass_through_node_index;
    }

    let dac_node_index = DACNode::new().add_to_context(&mut audio_context).await.unwrap();
    audio_context
        .connect(prev_node_index, dac_node_index).await
        .unwrap();

    audio_context.initialize_dac_from_defaults().await.unwrap();
    audio_context.play_stream().unwrap();

    let sample_rate = audio_context.sample_rate().unwrap();
    // sine_node.set_sample_rate(sample_rate).set_frequency(440.0);

    println!("{:?}", sample_rate);

    tokio::time::sleep(Duration::from_millis(3000)).await;

    audio_context.send_message(0);

    // audio_context.send_message(1);

    tokio::time::sleep(Duration::MAX).await;
}
