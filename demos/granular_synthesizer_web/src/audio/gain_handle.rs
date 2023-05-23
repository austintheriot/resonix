use crate::audio::gain::Gain;
use std::sync::{Arc, Mutex};

use super::{bump_counter::BumpCounter, gain_action::GainAction};

/// Wrapper around `Gain`, which makes it possible to access
/// the data from the audio thread, while also updating the value from the UI.
#[derive(Clone, Debug)]
pub struct GainHandle {
    gain: Arc<Mutex<Gain>>,
    counter: u32,
}

/// `GainHandle` re-implements all logic that `Gain` itself can perform
impl GainAction for GainHandle {
    const GAIN_MIN: f32 = Gain::GAIN_MIN;
    const GAIN_MAX: f32 = Gain::GAIN_MAX;

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

impl BumpCounter for GainHandle {
    fn bump_counter(&mut self) {
        self.counter = self.counter.wrapping_add(1);
    }
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
