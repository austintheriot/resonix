use crate::audio::gain::Gain;
use std::sync::{Arc, Mutex};

pub const GAIN_MIN: f32 = -1.0;
pub const GAIN_MAX: f32 = 1.0;

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

impl GainHandle {
    /// Bumps up counter so that Yew knows interanal state has changed,
    /// even when the internal ```gain``` points to the same memory
    /// 
    /// (i.e. now the external handle wrapper has a different count than
    /// handle that it was cloned from, so they will no longer be ==)
    fn bump_counter(&mut self) {
        self.counter = self.counter.wrapping_add(1);
    }

    fn sanitize_gain(input_gain: f32) -> f32 {
        input_gain.max(GAIN_MIN).min(GAIN_MAX)
    }

    pub fn new(gain: f32) -> Self {
        GainHandle {
            gain: Arc::new(Mutex::new(Gain::new(GainHandle::sanitize_gain(gain)))),
            counter: Default::default(),
        }
    }

    pub fn get(&self) -> f32 {
        self.gain.lock().unwrap().0
    }

    pub fn set(&mut self, gain: f32) {
        self.gain.lock().unwrap().0 = GainHandle::sanitize_gain(gain);
        self.bump_counter();
    }
}
