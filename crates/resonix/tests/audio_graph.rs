#[cfg(all(feature = "dac", feature = "mock_dac"))]
#[tokio::test]
async fn set_up_graph_then_initialize_audio() {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use resonix::{AudioContext, ConstantNode, DACNode, PassThroughNode};

    let mut audio_context = AudioContext::new();
    let constant_node = ConstantNode::new_with_signal_value(0.5);
    let constant_node_handle = audio_context.add_node(constant_node).await.unwrap();
    let pass_through_node = PassThroughNode::new();
    let pass_through_node_handle = audio_context.add_node(pass_through_node).await.unwrap();
    audio_context
        .connect(&constant_node_handle, &pass_through_node_handle)
        .await
        .unwrap();

    let dac_node = DACNode::new();
    let dac_node_handle = audio_context.add_node(dac_node).await.unwrap();
    audio_context
        .connect(&pass_through_node_handle, &dac_node_handle)
        .await
        .unwrap();

    let data_written = Arc::new(Mutex::new(Vec::new()));
    audio_context
        .initialize_dac_from_defaults(Arc::clone(&data_written))
        .unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;

    let data_written = data_written.lock().unwrap();
    assert_eq!(data_written[0..100], [0.5; 100])
}
