#[cfg(all(test, feature = "mock_test_dac"))]

#[tokio::test]
async fn example() {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use resonix::{AudioContext, ConstantNode, DACNode, PassThroughNode};

    let mut audio_context = AudioContext::new();
    let constant_node = ConstantNode::new_with_signal_value(1.0);
    let sine_node_handle = audio_context.add_node(constant_node).await.unwrap();
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

    let data_written = Arc::new(Mutex::new(Vec::new()));
    audio_context
        .initialize_dac_from_defaults(data_written)
        .unwrap();
    audio_context.play_stream().unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;

    assert!(!data_written.lock().unwrap().is_empty())
}
