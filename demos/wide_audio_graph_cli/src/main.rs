use std::time::Duration;

use resonix::{
    AudioContext, ConstantNode, DACNode, DownmixNode, Downmixer, MulticoreNode, MultiplyNode,
    PassThroughNode, SineNode,
};

/// Stress-tests a "wide" audio graph, where there is a lot of opportunity for parallelization

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    const NUM_PASS_THROUGH_NODES: u32 = 2000;

    let mut audio_context = AudioContext::new();
    let sine_node_handle = audio_context.add_node(SineNode::new(1, 440.0)).unwrap();
    let multicore_node_handle = audio_context.add_node(MulticoreNode::new(1000)).unwrap();
    let downmix_node_handle = audio_context
        .add_node(DownmixNode::new(1000, 2, Downmixer::Simple))
        .unwrap();
    let dac_node_handle = audio_context.add_node(DACNode::new(2)).unwrap();
    let constant_node_handle = audio_context
        .add_node(ConstantNode::new(2, 1.0 / NUM_PASS_THROUGH_NODES as f32))
        .unwrap();
    let multiply_node_handle = audio_context.add_node(MultiplyNode::new(2)).unwrap();

    // pass sine_node through many different intermediate nodes simultaneously
    for _ in 0..NUM_PASS_THROUGH_NODES {
        let pass_through_node_handle = audio_context.add_node(PassThroughNode::new(1)).unwrap();
        audio_context
            .connect(sine_node_handle, pass_through_node_handle)
            .unwrap();
        audio_context
            .connect(pass_through_node_handle, multicore_node_handle)
            .unwrap();
    }

    // convert many connections into a single, multichannel connection
    audio_context
        .connect(multicore_node_handle, downmix_node_handle)
        .unwrap();

    // scale down the audio
    audio_context
        .connect_with_indexes(downmix_node_handle, multiply_node_handle, 0, 0)
        .unwrap();
    audio_context
        .connect_with_indexes(constant_node_handle, multiply_node_handle, 0, 1)
        .unwrap();

    audio_context
        .connect(multiply_node_handle, dac_node_handle)
        .unwrap();

    let mut audio_context = audio_context.into_audio_init().unwrap();
    audio_context.play_stream().unwrap();

    tokio::time::sleep(Duration::MAX).await;
}
