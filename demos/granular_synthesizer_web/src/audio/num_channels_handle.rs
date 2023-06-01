use super::{bump_counter::BumpCounter, num_channels_action::NumChannelsAction};
use resonix::{
    granular_synthesizer::GranularSynthesizer, granular_synthesizer::GranularSynthesizerAction,
    NumChannels,
};
use std::sync::{Arc, Mutex};

/// Wrapper around `NumChannels`, which makes it possible to access
/// the data from the audio thread, while also updating the value from the UI.
#[derive(Clone, Debug)]
pub struct NumChannelsHandle {
    num_channels: Arc<Mutex<NumChannels>>,
    counter: u32,
}

impl From<usize> for NumChannelsHandle {
    fn from(num_channels: usize) -> Self {
        NumChannelsHandle::new(num_channels)
    }
}

impl NumChannelsAction for NumChannelsHandle {
    const DEFAULT_CHANNELS: usize = GranularSynthesizer::DEFAULT_NUM_CHANNELS;

    fn new(num_channels: impl Into<usize>) -> Self {
        NumChannelsHandle {
            num_channels: Arc::new(Mutex::new(NumChannels::new(num_channels))),
            counter: Default::default(),
        }
    }

    fn get(&self) -> NumChannels {
        self.num_channels.lock().unwrap().get().into()
    }

    fn set(&mut self, num_channels: impl Into<usize>) {
        self.num_channels.lock().unwrap().set(num_channels);
        self.bump_counter();
    }
}

impl BumpCounter for NumChannelsHandle {
    fn bump_counter(&mut self) {
        self.counter = self.counter.wrapping_add(1);
    }
}

impl PartialEq for NumChannelsHandle {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get() && self.counter == other.counter
    }
}

impl Default for NumChannelsHandle {
    fn default() -> Self {
        Self {
            num_channels: Arc::new(Mutex::new(NumChannels::default())),
            counter: Default::default(),
        }
    }
}
