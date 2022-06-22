use crate::audio::gain::Gain;
use std::sync::{Arc, Mutex};

use super::gain_action::GainAction;

/// Wrapper around `Gain`, which makes it possible to access
/// the data from the audio thread, while also updating the value from the UI.
#[derive(Clone, Debug)]
pub struct GainHandle {
    gain: Arc<Mutex<Gain>>,
    counter: u32,
}

impl PartialEq for GainHandle {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get() && self.counter == other.counter
    }
}

impl Default for GainHandle {
    fn default() -> Self {
        Self {
            gain: Arc::new(Mutex::new(Gain::default())),
            counter: Default::default(),
        }
    }
}

/// `GainHandle` re-implements all logic that `Gain` itself can perform
impl GainAction for GainHandle {
    fn get(&self) -> f32 {
        self.gain.lock().unwrap().get()
    }

    fn set(&mut self, gain: f32) {
        self.gain.lock().unwrap().set(gain);
        self.bump_counter();
    }

    fn new(gain: f32) -> Self {
        GainHandle {
            gain: Arc::new(Mutex::new(Gain::new(gain))),
            counter: Default::default(),
        }
    }
}

impl GainHandle {
    /// Bumps up counter so that Yew knows interanal state has changed,
    /// even when the internal ```gain``` points to the same memory
    /// 
    /// (i.e. now the external handle wrapper has a different count than
    /// handle that it was cloned from, so they will no longer be ==)
    fn bump_counter(&mut self) {
        self.counter = self.counter.wrapping_add(1);
    }
}
