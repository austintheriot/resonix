#[cfg(all(feature = "dac", feature = "mock_dac"))]
#[tokio::test]
async fn set_up_graph_then_initialize_audio() {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use resonix::{AudioContext, ConstantNode, DACNode, PassThroughNode};

    let mut audio_context = AudioContext::new();

    // set up audio graph immediately
    let constant_node = ConstantNode::new(2, 0.5);
    let constant_node_handle = audio_context.add_node(constant_node).unwrap();
    let pass_through_node = PassThroughNode::new(2);
    let pass_through_node_handle = audio_context.add_node(pass_through_node).unwrap();
    audio_context
        .connect(&constant_node_handle, &pass_through_node_handle)
        .unwrap();
    let dac_node = DACNode::new(2);
    let dac_node_handle = audio_context.add_node(dac_node).unwrap();
    audio_context
        .connect(&pass_through_node_handle, &dac_node_handle)
        .unwrap();

    // then set up audio out
    let data_written = Arc::new(Mutex::new(Vec::new()));
    let audio_context = audio_context
        .into_audio_init(Arc::clone(&data_written))
        .unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;

    // data is written to audio out
    {
        let data_written = data_written.lock().unwrap();
        assert_eq!(data_written[0..100], [0.5; 100])
    }
}

#[cfg(all(feature = "dac", feature = "mock_dac"))]
#[tokio::test]
async fn initialize_audio_then_set_up_graph() {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use resonix::{AudioContext, ConstantNode, DACNode, PassThroughNode};

    let data_written = Arc::new(Mutex::new(Vec::new()));

    // set up audio graph immediately
    let mut audio_context = AudioContext::new()
        .into_audio_init(Arc::clone(&data_written))
        .unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;

    // only silence being written to audio out
    {
        let data_written = data_written.lock().unwrap();
        assert_eq!(data_written[(data_written.len() - 100)..], [0.0; 100])
    }

    // set up audio graph
    let constant_node = ConstantNode::new(2, 0.5);
    let constant_node_handle = audio_context.add_node(constant_node).await.unwrap();
    let pass_through_node = PassThroughNode::new(2);
    let pass_through_node_handle = audio_context.add_node(pass_through_node).await.unwrap();
    audio_context
        .connect(&constant_node_handle, &pass_through_node_handle)
        .await
        .unwrap();
    let dac_node = DACNode::new(2);
    let dac_node_handle = audio_context.add_node(dac_node).await.unwrap();
    audio_context
        .connect(&pass_through_node_handle, &dac_node_handle)
        .await
        .unwrap();

    // now audio has been written to audio out
    tokio::time::sleep(Duration::from_secs(1)).await;
    {
        let data_written = data_written.lock().unwrap();
        assert_eq!(data_written[(data_written.len() - 100)..], [0.5; 100])
    }
}

#[cfg(all(feature = "dac", feature = "mock_dac"))]
#[tokio::test]
async fn updates_node_in_audio_thread() {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use resonix::{AudioContext, ConstantNode, DACNode, PassThroughNode};

    let mut audio_context = AudioContext::new();

    // set up audio graph
    let constant_node = ConstantNode::new(2, 0.5);
    let constant_node_handle = audio_context.add_node(constant_node).unwrap();
    let dac_node = DACNode::new(2);
    let dac_node_handle = audio_context.add_node(dac_node).unwrap();
    audio_context
        .connect(&constant_node_handle, &dac_node_handle)
        .unwrap();

    // start audio
    let data_written = Arc::new(Mutex::new(Vec::new()));
    let mut audio_context = audio_context
        .into_audio_init(Arc::clone(&data_written))
        .unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;

    // value of 0.5 being output
    {
        let data_written = data_written.lock().unwrap();
        assert_eq!(data_written[data_written.len() - 100..], [0.5; 100])
    }

    constant_node_handle
        .set_signal_value_async(&mut audio_context, 1.0)
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;

    // value of 1.0 being output
    {
        let data_written = data_written.lock().unwrap();
        assert_eq!(data_written[data_written.len() - 100..], [1.0; 100])
    }
}
