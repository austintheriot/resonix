use super::bump_counter::BumpCounter;
use std::sync::{Arc, Mutex};

/// Wrapper around `u32`, which makes it possible to access
/// the data from the audio thread, while also updating the value from the UI.
#[derive(Clone, Debug)]
pub struct GrainLenHandle {
    grain_len: Arc<Mutex<f32>>,
    counter: u32,
}

impl From<f32> for GrainLenHandle {
    fn from(grain_len: f32) -> Self {
        GrainLenHandle::new(grain_len)
    }
}

impl GrainLenHandle {
    pub fn get(&self) -> f32 {
        *self.grain_len.lock().unwrap()
    }

    pub fn set(&mut self, grain_len: f32) {
        *self.grain_len.lock().unwrap() = grain_len;
        self.bump_counter();
    }

    pub fn new(grain_len: f32) -> Self {
        GrainLenHandle {
            grain_len: Arc::new(Mutex::new(grain_len)),
            counter: Default::default(),
        }
    }
}

impl BumpCounter for GrainLenHandle {
    fn bump_counter(&mut self) {
        self.counter = self.counter.wrapping_add(1);
    }
}

impl PartialEq for GrainLenHandle {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get() && self.counter == other.counter
    }
}

impl Default for GrainLenHandle {
    fn default() -> Self {
        Self {
            grain_len: Arc::new(Mutex::new(Default::default())),
            counter: Default::default(),
        }
    }
}
