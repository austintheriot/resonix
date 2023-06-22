use std::{
    any::Any,
    collections::{HashSet, VecDeque},
    hash::{Hash, Hasher},
    ptr::addr_of,
    sync::{mpsc::Sender, Arc}, ops::{Deref, DerefMut},
};

#[cfg(feature = "dac")]
use cpal::{traits::StreamTrait, PauseStreamError, PlayStreamError};
use petgraph::{
    stable_graph::NodeIndex,
    visit::{Dfs, IntoNodeIdentifiers},
    Direction, Graph,
};
#[cfg(feature = "dac")]
use resonix_dac::{DACBuildError, DACConfig, DAC};
use uuid::Uuid;

use crate::{BoxedNode, Connection, DACNode, Node, NodeType, Processor};

#[derive(thiserror::Error, Debug)]
pub enum ConnectError {
    #[error("Node could not be found in the audio graph for index {node_index:?}. Are you sure you added it?")]
    NodeNotFound { node_index: NodeIndex },
    #[error("Node connection from {parent_node_name:?} to {child_node_name:?} failed. Expected `from_index` to be a max of {expected_from_index:?} and `to_index`  to be a max of {expected_to_index:?}. Received `from_index`  of {from_index:?} and `to_index` of {to_index:?}")]
    IncorrectIndex {
        expected_from_index: usize,
        expected_to_index: usize,
        from_index: usize,
        to_index: usize,
        parent_node_name: String,
        child_node_name: String,
    },
}

#[cfg(test)]
mod test_audio_context_inner {
    

    #[test]
    fn allows_running_audio_graph() {
        todo!()
        // let mut audio_context = AudioContext::default();
        // let constant_node_left = ConstantNode::new_with_signal_value(&mut audio_context, 4.0);
        // let constant_node_right = ConstantNode::new_with_signal_value(&mut audio_context, 0.5);

        // let pass_through_node_left = PassThroughNode::new(&mut audio_context);
        // constant_node_left.connect(&pass_through_node_left).unwrap();
        // let pass_through_node_right = PassThroughNode::new(&mut audio_context);
        // constant_node_right
        //     .connect(&pass_through_node_right)
        //     .unwrap();

        // let multiply_node = MultiplyNode::new(&mut audio_context);
        // pass_through_node_left.connect(&multiply_node).unwrap();
        // pass_through_node_right
        //     .connect_nodes_with_indexes(0, &multiply_node, 1)
        //     .unwrap();
        // let record_node = RecordNode::new(&mut audio_context);
        // multiply_node.connect(&record_node).unwrap();
        // audio_context.run();

        // // recording should now contain one sample
        // {
        //     let record_data = record_node.data();
        //     assert_eq!(record_data.len(), 1);
        //     assert_eq!(*record_data.first().unwrap(), 2.0);
        // }

        // audio_context.run();

        // // another sample should be recorded (with the same value)
        // {
        //     let record_data = record_node.data();
        //     assert_eq!(record_data.len(), 2);
        //     assert_eq!(*record_data.get(1).unwrap(), 2.0);
        // }
    }

    #[test]
    fn allows_getting_input_nodes() {
        todo!()
        // let mut audio_context = AudioContext::new();
        // let sine_node = SineNode::new(&mut audio_context);
        // RecordNode::new(&mut audio_context);
        // PassThroughNode::new(&mut audio_context);
        // let constant_node = ConstantNode::new(&mut audio_context);
        // MultiplyNode::new(&mut audio_context);

        // let input_nodes = audio_context.input_nodes();

        // assert_eq!(input_nodes.len(), 2);
        // assert!(input_nodes
        //     .iter()
        //     .any(|node| node.uuid() == sine_node.uuid()));
        // assert!(input_nodes
        //     .iter()
        //     .any(|node| node.uuid() == constant_node.uuid()));
    }

    #[cfg(feature = "dac")]
    #[test]
    fn allows_getting_dac_nodes() {
        todo!()
        // use crate::DACNode;

        // let mut audio_context = AudioContext::new();
        // SineNode::new(&mut audio_context);
        // RecordNode::new(&mut audio_context);
        // PassThroughNode::new(&mut audio_context);
        // ConstantNode::new(&mut audio_context);
        // MultiplyNode::new(&mut audio_context);
        // let dac_node = DACNode::new(&mut audio_context);

        // let dac_nodes = audio_context.dac_nodes();

        // assert_eq!(dac_nodes.len(), 1);
        // assert!(dac_nodes.iter().any(|node| node.uuid() == dac_node.uuid()));
    }
}

/// Cloning the audio context is an outward clone of the
/// audio context handle
#[derive(Debug)]
pub struct AudioContext {
    processor: Processor,
    uuid: Uuid,
    #[cfg(feature = "dac")]
    dac: Option<DAC>,
    tx: Option<Sender<usize>>,
}

unsafe impl Send for AudioContext {}
unsafe impl Sync for AudioContext {}

impl AudioContext {
    pub fn new() -> Self {
        Default::default()
    }

    #[cfg(feature = "dac")]
    pub fn num_channels(&self) -> Option<u16> {
        self.dac
            .as_ref()
            .map(|dac| dac.config.stream_config.channels)
    }

    #[cfg(feature = "dac")]
    pub fn sample_rate(&self) -> Option<u32> {
        self.dac
            .as_ref()
            .map(|dac| dac.config.stream_config.sample_rate.0)
    }

    #[cfg(feature = "dac")]
    pub async fn initialize_dac_from_defaults(&mut self) -> Result<&mut Self, DACBuildError> {
        self.initialize_dac_from_config(DACConfig::from_defaults()?)
            .await
    }

    pub fn send_message(&self, int: usize) {
        if let Some(tx) = &self.tx {
            tx.send(int).unwrap();
        }
    }

    #[cfg(feature = "dac")]
    pub async fn initialize_dac_from_config(
        &mut self,
        dac_config: DACConfig,
    ) -> Result<&mut Self, DACBuildError> {
        let (tx, rx) = std::sync::mpsc::channel();
        self.tx.replace(tx);
        let mut processor = self.processor.clone();
        let dac = DAC::from_dac_config(
            dac_config,
            move |buffer: &mut [f32], config: Arc<DACConfig>| {
                let num_channels = config.stream_config.channels as usize;

                if let Ok(message) = rx.try_recv() {
                    println!("message received: {message:?}");
                }

                for frame in buffer.chunks_mut(num_channels) {
                    processor.run();
                    let dac_nodes_sum = processor.dac_nodes_sum();
                    for channel in frame.iter_mut() {
                        *channel = cpal::Sample::from::<f32>(&dac_nodes_sum);
                    }
                }
            },
        )
        .await?;

        self.dac.replace(dac);

        Ok(self)
    }

    #[cfg(feature = "dac")]
    pub fn uninitialize_dac(&mut self) -> Option<DAC> {
        self.dac.take()
    }

    #[cfg(feature = "dac")]
    pub fn play_stream(&mut self) -> Result<(), PlayStreamError> {
        if let Some(dac) = &self.dac {
            return dac.stream.play();
        }

        Ok(())
    }

    #[cfg(feature = "dac")]
    pub fn pause_stream(&mut self) -> Result<(), PauseStreamError> {
        if let Some(dac) = &self.dac {
            return dac.stream.pause();
        }

        Ok(())
    }
}

impl PartialEq for AudioContext {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for AudioContext {}

impl PartialOrd for AudioContext {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.uuid.partial_cmp(&other.uuid)
    }
}

impl Ord for AudioContext {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uuid.cmp(&other.uuid)
    }
}

impl Hash for AudioContext {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}

impl Default for AudioContext {
    fn default() -> Self {
        Self {
            processor: Default::default(),
            uuid: Uuid::new_v4(),
            #[cfg(feature = "dac")]
            dac: Default::default(),
            tx: None,
        }
    }
}

impl Deref for AudioContext {
    type Target = Processor;

    fn deref(&self) -> &Self::Target {
        &self.processor
    }
}

impl DerefMut for AudioContext {
    fn deref_mut(&mut self) -> &mut Processor {
        &mut self.processor
    }
}

#[cfg(test)]
mod test_audio_context {}
