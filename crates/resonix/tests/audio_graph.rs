use std::any::Any;

use resonix_core::NumChannels;
use resonix_graph::{NodeType, NodeUid};

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

#[cfg(all(feature = "dac", feature = "mock_dac"))]
#[tokio::test]
async fn allows_implementing_custom_node() {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use resonix::{AudioContext, ConstantNode, DACNode, PassThroughNode};
    use resonix_dac::DACConfig;
    use resonix_graph::Node;

    let mut audio_context = AudioContext::new();

    #[derive(Debug, Clone)]
    struct OnOffNode {
        uid: u32,
        on: bool,
        num_outgoing_channels: NumChannels,
    }

    impl Node for OnOffNode {
        fn process(
            &mut self,
            inputs: &mut dyn Iterator<Item = std::cell::Ref<resonix_graph::Connection>>,
            outputs: &mut dyn Iterator<Item = std::cell::RefMut<resonix_graph::Connection>>,
        ) {
            let value = if self.on {
                1.0
            } else {
                0.0
            };

            while let Some(mut output) = outputs.next() {
                let output_data = output.data_mut();
                output_data.fill(value)
            }

            self.on = !self.on;
        }

        fn node_type(&self) -> NodeType {
            NodeType::Input
        }

        fn num_input_connections(&self) -> usize {
            0
        }

        fn num_output_connections(&self) -> usize {
            1
        }

        fn num_incoming_channels(&self) -> resonix_core::NumChannels {
            NumChannels::from(0)
        }

        fn num_outgoing_channels(&self) -> resonix_core::NumChannels {
            self.num_outgoing_channels
        }

        fn uid(&self) -> NodeUid {
            self.uid
        }

        fn set_uid(&mut self, uid: NodeUid) {
            self.uid = uid;
        }

        fn name(&self) -> String {
            String::from("OnOffNode")
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self as &dyn Any
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self as &mut dyn Any
        }

        fn requires_audio_updates(&self) -> bool {
            false
        }

        fn update_from_dac_config(&mut self, dac_config: Arc<resonix_dac::DACConfig>) {}
    }

    // set up audio graph
    let on_off_node = OnOffNode {
        num_outgoing_channels: NumChannels::from(1),
        on: true,
        uid: 0,
    };
    let on_off_node_handle = audio_context.add_node(on_off_node).unwrap();

    let dac_node = DACNode::new(1);
    let dac_node_handle = audio_context.add_node(dac_node).unwrap();
    audio_context
        .connect(&on_off_node_handle, &dac_node_handle)
        .unwrap();

    // create audio thread
    let data_written = Arc::new(Mutex::new(Vec::new()));
    let audio_context = audio_context
        .into_audio_init_from_config(
            Arc::new(DACConfig {
                num_channels: 1,
                sample_rate: 44100,
                num_frames: 10
            }),
            Arc::clone(&data_written),
        )
        .unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;

    // data is written to audio out
    {
        let data_written = data_written.lock().unwrap();
        assert_eq!(
            data_written.as_slice(),
            &[1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0,]
        )
    }
}
