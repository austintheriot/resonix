use super::{bump_counter::BumpCounter, channels::Channels, channels_action::ChannelsAction};
use audio::{
    granular_synthesizer::GranularSynthesizer,
    granular_synthesizer_action::GranularSynthesizerAction, percentage::Percentage,
};
use std::sync::{Arc, Mutex};

/// Wrapper around `Channels`, which makes it possible to access
/// the data from the audio thread, while also updating the value from the UI.
#[derive(Clone, Debug)]
pub struct ChannelsHandle {
    channels: Arc<Mutex<Channels>>,
    counter: u32,
}

impl From<f32> for ChannelsHandle {
    fn from(channels: f32) -> Self {
        ChannelsHandle::new(channels)
    }
}

impl ChannelsAction for ChannelsHandle {
    const DEFAULT_CHANNELS: f32 = GranularSynthesizer::DEFAULT_CHANNELS;

    fn new(channels: impl Into<Percentage>) -> Self {
        ChannelsHandle {
            channels: Arc::new(Mutex::new(Channels::new(channels))),
            counter: Default::default(),
        }
    }

    fn get(&self) -> Percentage {
        self.channels.lock().unwrap().get()
    }

    fn set(&mut self, channels: impl Into<Percentage>) {
        self.channels.lock().unwrap().set(channels);
        self.bump_counter();
    }
}

impl BumpCounter for ChannelsHandle {
    fn bump_counter(&mut self) {
        self.counter = self.counter.wrapping_add(1);
    }
}

impl PartialEq for ChannelsHandle {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get() && self.counter == other.counter
    }
}

impl Default for ChannelsHandle {
    fn default() -> Self {
        Self {
            channels: Arc::new(Mutex::new(Channels::default())),
            counter: Default::default(),
        }
    }
}
