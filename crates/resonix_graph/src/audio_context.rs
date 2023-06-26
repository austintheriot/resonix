use std::{
    hash::{Hash, Hasher},
    sync::Arc, collections::HashMap, marker::PhantomData,
};

use async_channel::{Receiver, Sender};
#[cfg(feature = "dac")]
use cpal::{traits::StreamTrait, PauseStreamError, PlayStreamError};
use log::{error, info};
use petgraph::{stable_graph::EdgeIndex, stable_graph::NodeIndex};
#[cfg(feature = "dac")]
use resonix_dac::{DACBuildError, DACConfig, DACConfigBuildError, DAC};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    AddNodeError, BoxedNode, ConnectError, ProcessorMessageRequest, ProcessorMessageResponse, Node, Processor, messages::{NodeMessageRequest, NodeMessageResponse}, NodeHandle,
};

#[cfg(feature = "dac")]
#[derive(Error, Debug)]
pub enum DacInitializeError {
    #[error("Error occurred while initializing DAC for the audio context because there is no Processor in the Audio Context. This indicates that the DAC has already been set up.")]
    NoProcessor,
    #[error("Error occurred while initializing DAC for the audio context. Original error: {0:?}")]
    DACBuildError(#[from] DACBuildError),
    #[error("Error occurred while initializing DAC for the audio context. Original error: {0:?}")]
    DACConfigBuildError(#[from] DACConfigBuildError),
}

#[derive(Debug)]
pub struct AudioContext {
    processor: Option<Processor>,
    #[cfg(feature = "dac")]
    dac: Option<DAC>,
    /// sends message to Processor once it has been moved into the audio thread
    processor_request_tx: Option<Sender<ProcessorMessageRequest<BoxedNode>>>,
    /// receives messages from Processor once it has been moved into the audio thread
    processor_response_rx: Option<Receiver<ProcessorMessageResponse>>,
    node_request_tx: Sender<NodeMessageRequest>,
    /// this receiver is moved into audio thread once it is initialized
    node_request_rx: Option<Receiver<NodeMessageRequest>>,
    node_response_txs: HashMap<Uuid, Sender<NodeMessageResponse>>,
    uuid: Uuid,
    request_id: u32,
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
    pub async fn initialize_dac_from_defaults(&mut self) -> Result<&mut Self, DacInitializeError> {
        self.initialize_dac_from_config(DACConfig::from_defaults()?)
            .await
    }

    #[cfg(feature = "dac")]
    pub async fn initialize_dac_from_config(
        &mut self,
        dac_config: DACConfig,
    ) -> Result<&mut Self, DacInitializeError> {
        let (audio_context_tx, processor_rx) = async_channel::unbounded();
        let (processor_tx, audio_context_rx) = async_channel::unbounded();
        self.processor_request_tx.replace(audio_context_tx);
        self.processor_response_rx.replace(audio_context_rx);
        let mut processor = self
            .processor
            .take()
            .ok_or(DacInitializeError::NoProcessor)?;
        let node_request_rx = self.node_request_rx.take().expect("DAC was initialized but node_request_rx was `None`");

        let dac = DAC::from_dac_config(
            dac_config,
            move |buffer: &mut [f32], config: Arc<DACConfig>| {
                let num_channels = config.stream_config.channels as usize;

                // run any messages send from the main thread that are ready to be processed
                while let Ok(message) = processor_rx.try_recv() {
                    info!("processor message received in DAC loop: {message:?}");
                    let processor_tx = processor_tx.clone();
                    run_message(message, &mut processor, processor_tx);
                }

                while let Ok(message) = node_request_rx.try_recv() {
                    info!("node message received in DAC loop: {message:?}");
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

    pub fn processor(&self) -> Option<&Processor> {
        self.processor.as_ref()
    }

    pub fn processor_mut(&mut self) -> Option<&mut Processor> {
        self.processor.as_mut()
    }

    pub async fn add_node<N: Node + 'static>(
        &mut self,
        node: N,
    ) -> Result<NodeHandle<N>, AddNodeError> {
        let uuid = *node.uuid();
        if let Some(processor) = &mut self.processor {
            processor.add_node(node).map(self.map_add_node_response_to_handle(uuid))
        } else {
            let new_request_id = self.new_request_id();
            self.processor_request_tx
                .as_ref()
                .expect("If `processor` is `None`, then `tx` should be defined")
                .send(ProcessorMessageRequest::AddNode {
                    id: new_request_id,
                    node: Box::new(node),
                })
                .await
                .unwrap();

            // it's necessary to `await` for a return value here,
            // since, in Wasm, there are not actual, multiple threads
            // going on--because of this, if the main thread is not
            // yielded with an await, the audio thread will never run
            // the message that was sent and no response will come
            loop {
                // probably unnecessary to loop here, but just in case
                // messages get out of order, filtering by request id
                // ensures that the response matches the request
                match self.processor_response_rx.as_mut().unwrap().recv().await {
                    Ok(ProcessorMessageResponse::AddNode { id, result }) if id == new_request_id => {
                        break result.map(self.map_add_node_response_to_handle(uuid));
                    }
                    _ => continue,
                }
            }
        }
    }

    fn map_add_node_response_to_handle<'a, N: Node>(&'a mut self, uuid: Uuid) -> impl FnMut(NodeIndex) -> NodeHandle<N> + 'a {
         move |node_index| {
            let (node_response_tx, node_response_rx) = async_channel::unbounded();
            self.node_response_txs.insert(uuid, node_response_tx);
            NodeHandle::<N> {
                uuid,
                node_index,
                node_request_tx: self.node_request_tx.clone(),
                node_response_rx,
                node_type: PhantomData::default(),
            }
        }
    }

    pub async fn connect(
        &mut self,
        parent_node_index: impl AsRef<NodeIndex>,
        child_node_index: impl AsRef<NodeIndex>,
    ) -> Result<EdgeIndex, ConnectError> {
        if let Some(processor) = &mut self.processor {
            processor.connect(*parent_node_index.as_ref(), *child_node_index.as_ref())
        } else {
            let new_request_id = self.new_request_id();
            self.processor_request_tx
                .as_mut()
                .expect("If `processor` is `None`, then `tx` should be defined")
                .send(ProcessorMessageRequest::Connect {
                    id: new_request_id,
                    parent_node_index: *parent_node_index.as_ref(),
                    child_node_index: *child_node_index.as_ref(),
                })
                .await
                .unwrap();

            loop {
                match self.processor_response_rx.as_mut().unwrap().recv().await {
                    Ok(ProcessorMessageResponse::Connect { id, result }) if id == new_request_id => {
                        break result;
                    }
                    _ => continue,
                }
            }
        }
    }

    fn new_request_id(&mut self) -> u32 {
        self.request_id = self.request_id.wrapping_add(1);
        self.request_id
    }
}

fn run_message<N: Node + 'static>(
    message: ProcessorMessageRequest<N>,
    processor: &mut Processor,
    processor_tx: Sender<ProcessorMessageResponse>,
) {
    match message {
        ProcessorMessageRequest::AddNode { id, node } => {
            let result = processor.add_node(node);
            let response = ProcessorMessageResponse::AddNode { id, result };
            processor_tx.try_send(response).unwrap();
        }
        ProcessorMessageRequest::Connect {
            id,
            parent_node_index,
            child_node_index,
        } => {
            let result = processor.connect(parent_node_index, child_node_index);
            let response = ProcessorMessageResponse::Connect { id, result };
            processor_tx.try_send(response).unwrap();
        }
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
        let (node_request_tx, node_request_rx) = async_channel::unbounded();
        Self {
            processor: Some(Processor::default()),
            uuid: Uuid::new_v4(),
            #[cfg(feature = "dac")]
            dac: Default::default(),
            processor_request_tx: None,
            processor_response_rx: None,
            request_id: 0,
            node_request_tx,
            node_request_rx: Some(node_request_rx),
            node_response_txs: HashMap::default(),
        }
    }
}

#[cfg(test)]
mod test_audio_context {}
