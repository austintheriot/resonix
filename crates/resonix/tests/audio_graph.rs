#[cfg(all(feature = "dac", feature = "mock_dac"))]
#[tokio::test]
async fn test_constant_node_pass_though_to_dac() {
    use std::{
        sync::{Arc, Mutex},
        time::Duration, error::Error, fmt::Debug,
    };

    use resonix::{AudioContext, ConstantNode, DACNode, PassThroughNode};

    let mut audio_context = AudioContext::new();
    audio_context.play_stream().unwrap();
    
    let constant_node = ConstantNode::new(0.5);
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

    audio_context.run();

    let data_written = data_written.lock().unwrap();
    assert_eq!(data_written.len(), 2);
    assert_eq!(data_written[0..2], [0.5, 0.5]);
}

#[cfg(all(feature = "dac", feature = "mock_dac"))]
#[tokio::test]
async fn test_setting_up_audio_graph_after_starting_audio() {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use resonix::{AudioContext, ConstantNode, DACNode, PassThroughNode, SineNode};

    let mut audio_context = AudioContext::new();

    // initialize DAC right away
    let data_written = Arc::new(Mutex::new(Vec::new()));
    audio_context
        .initialize_dac_from_defaults(Arc::clone(&data_written))
        .unwrap();

    // no data yet
    {
        let data_written_lock = data_written.lock().unwrap();
        assert_eq!(data_written_lock.len(), 0);
    }

    audio_context.run();

    // only silence without audio graph initialized
    {
        let data_written_lock = data_written.lock().unwrap();
        assert_eq!(data_written_lock.len(), 2);
        assert_eq!(data_written_lock[0..2], [0.0, 0.0]);
    }

    // THEN create audio graph
    let sine_node = SineNode::new_with_config(audio_context.sample_rate().unwrap(), 440.0);
    let sine_node_handle = audio_context.add_node(sine_node).await.unwrap();
    let dac_node = DACNode::new();
    let dac_node_handle = audio_context.add_node(dac_node).await.unwrap();
    audio_context
        .connect(sine_node_handle, dac_node_handle)
        .await
        .unwrap();

    audio_context.run();

    let data_written = data_written.lock().unwrap();
    assert_eq!(data_written.len(), 2);
    assert_eq!(data_written[0..2], [0.5, 0.5])
}
