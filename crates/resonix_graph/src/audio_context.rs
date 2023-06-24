use std::{
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    sync::{
        mpsc::{Receiver, Sender},
        Arc,
    },
};

use async_trait::async_trait;
#[cfg(feature = "dac")]
use cpal::{traits::StreamTrait, PauseStreamError, PlayStreamError};

use log::{info, error};
use petgraph::stable_graph::NodeIndex;
use resonix_dac::DACConfigBuildError;
#[cfg(feature = "dac")]
use resonix_dac::{DACBuildError, DACConfig, DAC};
use thiserror::Error;
use uuid::Uuid;

#[cfg(target_arch = "wasm32")]
use std::time::Duration;
#[cfg(target_arch = "wasm32")]
use gloo_timers::future::sleep;

use crate::{ConnectError, Node, Processor};

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
    /// sends message to Processor once it moves into the audio thread
    #[cfg(feature = "dac")]
    tx: Option<Sender<usize>>,
    /// receives messages from Processor once it moves into the audio thread
    #[cfg(feature = "dac")]
    rx: Option<Receiver<usize>>,
    uuid: Uuid,
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

    pub fn send_message(&self, int: usize) {
        if let Some(tx) = &self.tx {
            tx.send(int).unwrap();
        }
    }

    #[cfg(feature = "dac")]
    pub async fn initialize_dac_from_config(
        &mut self,
        dac_config: DACConfig,
    ) -> Result<&mut Self, DacInitializeError> {
        let (mut audio_context_tx, mut processor_rx) = std::sync::mpsc::channel();
        let (mut processor_tx, mut audio_context_rx) = std::sync::mpsc::channel();
        self.tx.replace(audio_context_tx);
        self.rx.replace(audio_context_rx);
        let mut processor = self
            .processor
            .take()
            .ok_or(DacInitializeError::NoProcessor)?;

        let dac = DAC::from_dac_config(
            dac_config,
            move |buffer: &mut [f32], config: Arc<DACConfig>| {
                let num_channels = config.stream_config.channels as usize;

                // testing sending and receiving messages
                if let Ok(message) = processor_rx.try_recv() {
                    info!("message received: {message:?}");

                    let processor_tx = processor_tx.clone();

                    processor_tx.send(message).unwrap();
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

    pub async fn add_node<N: Node + 'static>(&mut self, node: N) -> Result<NodeIndex, N> {
        if let Some(processor) = &mut self.processor {
            processor.add_node(node).await
        } else {
            self.tx
                .as_ref()
                .expect("If `processor` is `None`, then `tx` should be defined")
                .send(1234)
                .unwrap();

            #[cfg(target_arch = "wasm32")]
            {
                sleep(Duration::from_millis(1000)).await;
            }

            if let Ok(message) = self.rx.as_mut().unwrap().try_recv() {
                info!(
                    "audio_context.add_log(): received back message from dac! {:?}",
                    message
                );
            } else {
                error!("audio_context.add_log(): could not get message back from dac!")
            }

            // todo : delete
            Ok(NodeIndex::new(0))
        }
    }

    pub async fn connect(
        &mut self,
        node_1: NodeIndex,
        node_2: NodeIndex,
    ) -> Result<&mut Self, ConnectError> {
        if let Some(processor) = &mut self.processor {
            processor.connect(node_1, node_2).await.unwrap();
            Ok(self)
        } else {
            self.tx
                .as_mut()
                .expect("If `processor` is `None`, then `tx` should be defined")
                .send(5678)
                .unwrap();
            let message = self.rx.as_mut().unwrap().try_recv().unwrap();
            info!("audio context received back message! {:?}", message);
            // todo : delete
            Ok(self)
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
        Self {
            processor: Some(Processor::default()),
            uuid: Uuid::new_v4(),
            #[cfg(feature = "dac")]
            dac: Default::default(),
            tx: None,
            rx: None,
        }
    }
}

#[cfg(test)]
mod test_audio_context {}
