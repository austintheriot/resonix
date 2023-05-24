use super::bump_counter::BumpCounter;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

#[derive(Clone, Debug)]
pub struct GrainLenHandle {
    grain_len: Arc<Mutex<Duration>>,
    counter: u32,
}

impl From<Duration> for GrainLenHandle {
    fn from(grain_len: Duration) -> Self {
        GrainLenHandle::new(grain_len)
    }
}

impl GrainLenHandle {
    pub fn get(&self) -> Duration {
        *self.grain_len.lock().unwrap()
    }

    pub fn set(&mut self, grain_len: impl Into<Duration>) {
        *self.grain_len.lock().unwrap() = grain_len.into();
        self.bump_counter();
    }

    pub fn new(grain_len: Duration) -> Self {
        GrainLenHandle {
            grain_len: Arc::new(Mutex::new(Duration::from(grain_len))),
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
