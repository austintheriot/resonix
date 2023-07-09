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
    messages::{MessageError, NodeMessageRequest, UpdateNodeError},
    AddNodeError, BoxedNode, ConnectError, Node, NodeHandle, Processor, ProcessorMessageRequest,
    ProcessorMessageResponse, SineNode, NodeUid,
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
        uid: NodeUid,
    ) -> impl FnMut((NodeUid, NodeIndex)) -> NodeHandle<N> + '_ {
        move |(uid, node_index)| NodeHandle::<N> {
            uid,
            node_type: PhantomData,
        }
    }
}

impl AudioContext<AudioUninit> {
    pub fn new() -> AudioContext<AudioUninit> {
        Default::default()
    }

    pub fn connect(
        &mut self,
        parent_node_uid: impl AsRef<NodeUid>,
        child_node_uid: impl AsRef<NodeUid>,
    ) -> Result<EdgeIndex, ConnectError> {
        self.processor
            .as_mut()
            .unwrap()
            .connect(*parent_node_uid.as_ref(), *child_node_uid.as_ref())
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

    pub(crate) fn handle_node_message_request(
        &mut self,
        node_message_request: NodeMessageRequest,
    ) -> Result<(), UpdateNodeError> {
        let processor = self.processor.as_mut().unwrap();
        processor.handle_node_message_request(node_message_request)
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
            uuid: self.uuid,
            request_id: self.request_id,
            audio_state: PhantomData,
        })
    }
}

/// When an asynchronous message is received for the processor 
/// in the audio thread, this function runs that message
/// on behalf of that processor synchronously and sends back a response
/// to the main thread
#[cfg(feature = "dac")]
fn run_processor_message<N: Node + 'static>(
    message: ProcessorMessageRequest<N>,
    processor: &mut Processor,
    processor_tx: Sender<ProcessorMessageResponse>,
    dac_config: Arc<DACConfig>,
) {
    let response = match message {
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
            ProcessorMessageResponse::AddNode {
                request_id: id,
                result,
            }
        }
        ProcessorMessageRequest::Connect {
            request_id: id,
            parent_node_uid,
            child_node_uid,
        } => {
            let result = processor.connect(parent_node_uid, child_node_uid);
            ProcessorMessageResponse::Connect {
                request_id: id,
                result,
            }
        }
        ProcessorMessageRequest::UpdateNode {
            request_id,
            request,
        } => {
            let result = processor.handle_node_message_request(request);
            ProcessorMessageResponse::UpdateNode { request_id, result }
        }
    };
    processor_tx.try_send(response).unwrap();
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

    /// Asynchronously updates node from the audio graph in the audio thread
    pub(crate) async fn handle_node_message_request(
        &mut self,
        node_message_request: NodeMessageRequest,
    ) -> Result<(), MessageError> {
        self.send_message_to_processor(
            |request_id| ProcessorMessageRequest::UpdateNode {
                request_id,
                request: node_message_request,
            },
            |node_message_response| {
                let ProcessorMessageResponse::UpdateNode { result, .. } = node_message_response else {
                return Err(MessageError::WrongResponseReceived)
            };

                result.map_err(|e| MessageError::from(e))
            },
        )
        .await
    }

    /// Asynchronously connect two nodes from the audio graph inside the audio thread
    pub async fn connect(
        &mut self,
        parent_node_uid: impl AsRef<NodeUid>,
        child_node_uid: impl AsRef<NodeUid>,
    ) -> Result<EdgeIndex, MessageError> {
        self.send_message_to_processor(
            |request_id| ProcessorMessageRequest::Connect {
                request_id,
                parent_node_uid: *parent_node_uid.as_ref(),
                child_node_uid: *child_node_uid.as_ref(),
            },
            |node_message_response| {
                let ProcessorMessageResponse::Connect { result, .. } = node_message_response else {
                return Err(MessageError::WrongResponseReceived)
            };

                result.map_err(|e| MessageError::from(e))
            },
        )
        .await
    }

    /// Asynchronously add a node to the audio graph inside the audio thread
    pub async fn add_node<N: Node + 'static>(
        &mut self,
        node: N,
    ) -> Result<NodeHandle<N>, MessageError> {
        self.send_message_to_processor(|request_id| ProcessorMessageRequest::AddNode {
            request_id,
            node: Box::new(node),
        }, |node_message_response| {
            let ProcessorMessageResponse::AddNode { result, request_id } = node_message_response else {
                return Err(MessageError::WrongResponseReceived)
            };

            return result.map_err(|e| MessageError::from(e)).map(|result| (result, request_id));
        })
        .await
        .map(|(result, request_id)| self.node_index_into_node_handle(request_id)(result))
    }

    async fn send_message_to_processor<R>(
        &mut self,
        create_request: impl FnOnce(u32) -> ProcessorMessageRequest<BoxedNode>,
        mut handle_response: impl FnMut(ProcessorMessageResponse) -> Result<R, MessageError>,
    ) -> Result<R, MessageError> {
        let new_request_id = self.new_request_id();
        self.processor_request_tx
            .as_mut()
            .expect("If `processor` is `None`, then `tx` should be defined")
            .send((create_request)(new_request_id))
            .await
            .unwrap();

        // it's necessary to `await` for a channel value here,
        // since, in Wasm, there are not actual, multiple threads
        // going on--because of this, if the main thread is not
        // yielded with an await, the audio thread will never run
        // the message that was sent and no response will come
        //
        // probably unnecessary to loop here, but just in case
        // messages get out of order, filtering by request id
        // ensures that the response matches the request
        while let Ok(response) = self.processor_response_rx.as_mut().unwrap().recv().await {
            let request_id = response.request_id();

            if request_id != new_request_id {
                continue;
            }

            return (handle_response)(response);
        }

        Err(MessageError::NoMatchingMessageReceived)
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
        Self {
            processor: Some(Processor::default()),
            uuid: Uuid::new_v4(),
            #[cfg(feature = "dac")]
            dac: Default::default(),
            processor_request_tx: None,
            processor_response_rx: None,
            request_id: 0,
            node_request_tx,
            audio_state: PhantomData,
        }
    }
}

#[cfg(test)]
mod test_audio_context {}
