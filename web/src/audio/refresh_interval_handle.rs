use common::{
    granular_synthesizer::GranularSynthesizer,
    granular_synthesizer_action::GranularSynthesizerAction,
};
use super::bump_counter::BumpCounter;
use std::sync::{Arc, Mutex};

/// Wrapper around `u32`, which makes it possible to access
/// the data from the audio thread, while also updating the value from the UI.
#[derive(Clone, Debug)]
pub struct RefreshIntervalHandle {
    refresh_interval: Arc<Mutex<u32>>,
    counter: u32,
}

impl From<u32> for RefreshIntervalHandle {
    fn from(refresh_interval: u32) -> Self {
        RefreshIntervalHandle::new(refresh_interval)
    }
}

impl RefreshIntervalHandle {
    pub fn get(&self) -> u32 {
        *self.refresh_interval.lock().unwrap()
    }

    pub fn set(&mut self, refresh_interval: u32) {
        *self.refresh_interval.lock().unwrap() =
            GranularSynthesizer::sanitize_refresh_interval(refresh_interval);
        self.bump_counter();
    }

    pub fn new(refresh_interval: u32) -> Self {
        RefreshIntervalHandle {
            refresh_interval: Arc::new(Mutex::new(GranularSynthesizer::sanitize_refresh_interval(
                refresh_interval,
            ))),
            counter: Default::default(),
        }
    }
}

impl BumpCounter for RefreshIntervalHandle {
    fn bump_counter(&mut self) {
        self.counter = self.counter.wrapping_add(1);
    }
}

impl PartialEq for RefreshIntervalHandle {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get() && self.counter == other.counter
    }
}

impl Default for RefreshIntervalHandle {
    fn default() -> Self {
        Self {
            refresh_interval: Arc::new(Mutex::new(Default::default())),
            counter: Default::default(),
        }
    }
}
