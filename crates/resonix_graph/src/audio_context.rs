use std::{
    hash::{Hash, Hasher},
    marker::PhantomData,
    sync::Arc,
};

use async_channel::{Receiver, Sender};
#[cfg(feature = "dac")]
use cpal::{traits::StreamTrait, PauseStreamError, PlayStreamError};
use log::{error, info};
use petgraph::{stable_graph::EdgeIndex, stable_graph::NodeIndex};
use resonix_core::SineInterface;
#[cfg(feature = "dac")]
use resonix_dac::{DACBuildError, DACConfig, DACConfigBuildError, DAC};
use thiserror::Error;
use uuid::Uuid;

#[cfg(feature = "mock_dac")]
use std::sync::Mutex;

use crate::{
    messages::{NodeMessageError, NodeMessageRequest, NodeMessageResponse},
    AddNodeError, BoxedNode, ConnectError, Node, NodeHandle, Processor, ProcessorMessageRequest,
    ProcessorMessageResponse, SineNode,
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

/// Zero-sized marker `AudioContext` that indicates that
/// the audio thread HAS been initialized.
///
/// When AudioContext is `AudioInit`, then the user
/// can access methods related to DAC-specific audio data,
/// such as sample rate, number of channels, etc.
///
/// This also means the audio graph itself has been moved into
/// the audio thread, and any further changes will have to be
/// done asynchronously.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AudioInit;

/// Zero-sized marker `AudioContext` that indicates that
/// the audio thread has NOT yet been initialized.
///
/// When AudioContext is `AudioUninit`, then the user
/// does NOT have access methods related to DAC-specific audio
/// data, such as sample rate, number of channels, etc.
///
/// When this is the case, that means the audio graph itself
/// hasn't yet been moved into the audio thread an
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AudioUninit;

#[derive(Debug)]
pub struct AudioContext<A = AudioUninit> {
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
    /// Cloned into `NodeHandle`s when they are created,
    /// allowing `NodeHandle`s to receive messages back from
    /// the audio thread
    node_response_rx: Receiver<NodeMessageResponse>,
    /// Allows sending messages from audio thread
    /// to `NodeHandle`s
    node_response_tx: Sender<NodeMessageResponse>,
    uuid: Uuid,
    request_id: u32,
    audio_state: PhantomData<A>,
}

impl<A> AudioContext<A> {
    pub fn processor(&self) -> Option<&Processor> {
        self.processor.as_ref()
    }

    pub fn processor_mut(&mut self) -> Option<&mut Processor> {
        self.processor.as_mut()
    }

    fn node_index_into_node_handle<N: Node>(
        &mut self,
        uid: u32,
    ) -> impl FnMut(NodeIndex) -> NodeHandle<N> + '_ {
        move |node_index| NodeHandle::<N> {
            uid,
            node_index,
            node_request_tx: self.node_request_tx.clone(),
            node_response_rx: self.node_response_rx.clone(),
            node_type: PhantomData,
        }
    }
}

#[cfg(feature = "dac")]
fn run_node_message(
    message: NodeMessageRequest,
    processor: &mut Processor,
    node_response_tx: &Sender<NodeMessageResponse>,
) {
    match message {
        NodeMessageRequest::SineSetFrequency {
            node_uid,
            node_index,
            new_frequency,
        } => {
            let result = set_sine_node_frequency(processor, node_index, node_uid, new_frequency);
            node_response_tx
                .try_send(NodeMessageResponse::SineSetFrequency { node_uid, result })
                .unwrap();
        }
    };
}

#[cfg(feature = "dac")]
fn set_sine_node_frequency(
    processor: &mut Processor,
    node_index: NodeIndex,
    uid: u32,
    new_frequency: f32,
) -> Result<(), NodeMessageError> {
    let node = processor
        .node_weight_mut(node_index)
        .ok_or(NodeMessageError::NodeNotFound { uid, node_index })?;
    let mut sine_node = node.borrow_mut();
    let sine_node = sine_node
        .as_any_mut()
        .downcast_mut::<SineNode>()
        .ok_or(NodeMessageError::WrongNodeType { uid, node_index })?;
    sine_node.set_frequency(new_frequency);
    Ok(())
}

#[cfg(feature = "dac")]
fn run_processor_message<N: Node + 'static>(
    message: ProcessorMessageRequest<N>,
    processor: &mut Processor,
    processor_tx: Sender<ProcessorMessageResponse>,
    dac_config: Arc<DACConfig>,
) {
    match message {
        ProcessorMessageRequest::AddNode {
            request_id: id,
            mut node,
        } => {
            let should_update = node.requires_audio_updates();
            if should_update {
                node.update_from_dac_config(dac_config);
            }

            let node = node;
            let result = processor.add_node(node);
            let response = ProcessorMessageResponse::AddNode { id, result };
            processor_tx.try_send(response).unwrap();
        }
        ProcessorMessageRequest::Connect {
            request_id: id,
            parent_node_index,
            child_node_index,
        } => {
            let result = processor.connect(parent_node_index, child_node_index);
            let response = ProcessorMessageResponse::Connect { id, result };
            processor_tx.try_send(response).unwrap();
        }
    }
}

impl AudioContext<AudioInit> {
    #[cfg(feature = "dac")]
    pub fn num_channels(&self) -> Option<u16> {
        self.dac.as_ref().map(|dac| dac.config.num_channels())
    }

    #[cfg(feature = "dac")]
    pub fn sample_rate(&self) -> Option<u32> {
        self.dac.as_ref().map(|dac| dac.config.sample_rate())
    }

    #[cfg(feature = "dac")]
    pub fn take_dac(&mut self) -> Option<DAC> {
        self.dac.take()
    }

    #[cfg(all(feature = "dac", not(feature = "mock_dac")))]
    pub fn play_stream(&mut self) -> Result<(), PlayStreamError> {
        self.dac.as_ref().unwrap().stream.play()?;

        Ok(())
    }

    #[cfg(all(feature = "dac", not(feature = "mock_dac")))]
    pub fn pause_stream(&mut self) -> Result<(), PauseStreamError> {
        if let Some(dac) = &self.dac {
            return dac.stream.pause();
        }

        Ok(())
    }

    fn new_request_id(&mut self) -> u32 {
        self.request_id = self.request_id.wrapping_add(1);
        self.request_id
    }

    pub async fn connect(
        &mut self,
        parent_node_index: impl AsRef<NodeIndex>,
        child_node_index: impl AsRef<NodeIndex>,
    ) -> Result<EdgeIndex, ConnectError> {
        let new_request_id = self.new_request_id();
        self.processor_request_tx
            .as_mut()
            .expect("If `processor` is `None`, then `tx` should be defined")
            .send(ProcessorMessageRequest::Connect {
                request_id: new_request_id,
                parent_node_index: *parent_node_index.as_ref(),
                child_node_index: *child_node_index.as_ref(),
            })
            .await
            .unwrap();

        while let Ok(response) = self.processor_response_rx.as_mut().unwrap().recv().await {
            let ProcessorMessageResponse::Connect { id, result } = response else {
                continue;
            };

            if id != new_request_id {
                continue;
            }

            return result;
        }

        Err(ConnectError::NoMatchingMessageReceived)
    }

    pub async fn add_node<N: Node + 'static>(
        &mut self,
        node: N,
    ) -> Result<NodeHandle<N>, AddNodeError> {
        let uid = node.uid();
        let new_request_id = self.new_request_id();
        self.processor_request_tx
            .as_ref()
            .expect("If `processor` is `None`, then `tx` should be defined")
            .send(ProcessorMessageRequest::AddNode {
                request_id: new_request_id,
                node: Box::new(node),
            })
            .await
            .unwrap();

        // it's necessary to `await` for a channel values here,
        // since, in Wasm, there are not actual, multiple threads
        // going on--because of this, if the main thread is not
        // yielded with an await, the audio thread will never run
        // the message that was sent and no response will come
        //
        // probably unnecessary to loop here, but just in case
        // messages get out of order, filtering by request id
        // ensures that the response matches the request
        while let Ok(response) = self.processor_response_rx.as_mut().unwrap().recv().await {
            let ProcessorMessageResponse::AddNode { id, result } = response else {
                continue;
            };

            if id != new_request_id {
                continue;
            }

            return result.map(self.node_index_into_node_handle(uid));
        }

        Err(AddNodeError::NoMatchingMessageReceived)
    }
}

impl AudioContext<AudioUninit> {
    pub fn new() -> AudioContext<AudioUninit> {
        Default::default()
    }

    pub fn connect(
        &mut self,
        parent_node_index: impl AsRef<NodeIndex>,
        child_node_index: impl AsRef<NodeIndex>,
    ) -> Result<EdgeIndex, ConnectError> {
        self.processor
            .as_mut()
            .unwrap()
            .connect(*parent_node_index.as_ref(), *child_node_index.as_ref())
    }

    pub fn add_node<N: Node + 'static>(&mut self, node: N) -> Result<NodeHandle<N>, AddNodeError> {
        let uid = node.uid();
        self.processor
            .as_mut()
            .unwrap()
            .add_node(node)
            .map(self.node_index_into_node_handle(uid))
    }

    /// Uses default audio configuration to create an audio thread
    #[cfg(all(feature = "dac"))]
    pub fn into_audio_init(
        self,
        #[cfg(feature = "mock_dac")] data_written: Arc<Mutex<Vec<f32>>>,
    ) -> Result<AudioContext<AudioInit>, (Self, DacInitializeError)> {
        let dac_config = match DACConfig::from_defaults() {
            Err(e) => return Err((self, DacInitializeError::from(e))),
            Ok(dac_config) => dac_config,
        };

        let dac_config = Arc::new(dac_config);

        self.into_audio_init_from_config(
            dac_config,
            #[cfg(feature = "mock_dac")]
            data_written,
        )
    }

    ///  Users user-specified audio configuration to create audio thread
    #[cfg(all(feature = "dac"))]
    pub fn into_audio_init_from_config(
        mut self,
        dac_config: Arc<DACConfig>,
        #[cfg(feature = "mock_dac")] data_written: Arc<Mutex<Vec<f32>>>,
    ) -> Result<AudioContext<AudioInit>, (Self, DacInitializeError)> {
        use resonix_core::NumChannels;

        let (audio_context_tx, processor_rx) = async_channel::unbounded();
        let (processor_tx, audio_context_rx) = async_channel::unbounded();
        self.processor_request_tx.replace(audio_context_tx);
        self.processor_response_rx.replace(audio_context_rx);
        let processor = self.processor.take().ok_or(DacInitializeError::NoProcessor);

        let mut processor = match processor {
            Err(e) => return Err((self, e)),
            Ok(processor) => processor,
        };

        let node_request_rx = self
            .node_request_rx
            .take()
            .expect("DAC was initialized but node_request_rx was `None`");
        let node_response_tx = self.node_response_tx.clone();
        let mut initial_audio_update_has_been_run = false;
        let dac_result = DAC::from_dac_config(
            Arc::clone(&dac_config),
            move |buffer: &mut [f32], config: Arc<DACConfig>| {
                let num_audio_channels_out = NumChannels::from(config.num_channels());

                // the first time the audio loop is run, all nodes that require
                // dac-specific audio data must be updated.
                // all subsequent updates to nodes that are added to the
                // audio graph occur at a fine-grained level when the node
                // is added to the graph
                if !initial_audio_update_has_been_run {
                    initial_audio_update_has_been_run = true;
                    processor.update_audio_nodes(Arc::clone(&dac_config));
                }

                // run any messages for the processor, sent from the main thread, that are ready to be processed
                while let Ok(message) = processor_rx.try_recv() {
                    info!("processor message received in DAC loop: {message:?}");
                    let processor_tx = processor_tx.clone();
                    run_processor_message(
                        message,
                        &mut processor,
                        processor_tx,
                        Arc::clone(&dac_config),
                    );
                }

                // run any messages for individual nodes, sent from the main thread, that are ready to be processed
                while let Ok(message) = node_request_rx.try_recv() {
                    info!("node message received in DAC loop: {message:?}");
                    run_node_message(message, &mut processor, &node_response_tx);
                }

                // run audio graph and copy audio graph output information into actual audio-out buffer
                for frame in buffer.chunks_mut(*num_audio_channels_out) {
                    processor.run();
                    let dac_nodes_sum = processor.dac_nodes_sum(num_audio_channels_out);
                    for (channel, sum) in frame.iter_mut().zip(&dac_nodes_sum) {
                        *channel = cpal::Sample::from::<f32>(sum);
                    }
                }
            },
            #[cfg(feature = "mock_dac")]
            data_written,
        );

        let dac = match dac_result {
            Err(e) => return Err((self, DacInitializeError::from(e))),
            Ok(values) => values,
        };

        Ok(AudioContext::<AudioInit> {
            dac: Some(dac),

            // copy the rest of AudioUninit properties into new object
            processor: self.processor,
            processor_request_tx: self.processor_request_tx,
            processor_response_rx: self.processor_response_rx,
            node_request_tx: self.node_request_tx,
            node_request_rx: self.node_request_rx,
            node_response_rx: self.node_response_rx,
            node_response_tx: self.node_response_tx,
            uuid: self.uuid,
            request_id: self.request_id,
            audio_state: PhantomData,
        })
    }
}

impl<A> PartialEq for AudioContext<A> {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl<A> Eq for AudioContext<A> {}

impl<A> PartialOrd for AudioContext<A> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.uuid.partial_cmp(&other.uuid)
    }
}

impl<A> Ord for AudioContext<A> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uuid.cmp(&other.uuid)
    }
}

impl<A> Hash for AudioContext<A> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}

impl<A> Default for AudioContext<A> {
    fn default() -> Self {
        let (node_request_tx, node_request_rx) = async_channel::unbounded();
        let (node_response_tx, node_response_rx) = async_channel::unbounded();
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
            node_response_tx,
            node_response_rx,
            audio_state: PhantomData,
        }
    }
}

#[cfg(test)]
mod test_audio_context {}
