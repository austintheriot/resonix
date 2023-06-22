use std::{
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    sync::{mpsc::Sender, Arc},
};

#[cfg(feature = "dac")]
use cpal::{traits::StreamTrait, PauseStreamError, PlayStreamError};

#[cfg(feature = "dac")]
use resonix_dac::{DACBuildError, DACConfig, DAC};
use uuid::Uuid;

use crate::Processor;

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
