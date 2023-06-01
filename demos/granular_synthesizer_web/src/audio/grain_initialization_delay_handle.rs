use super::bump_counter::BumpCounter;
use resonix::{
    granular_synthesizer::GranularSynthesizer, granular_synthesizer::GranularSynthesizerAction,
};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

#[derive(Clone, Debug)]
pub struct GrainInitializationDelayHandle {
    grain_initialization_delay: Arc<Mutex<Duration>>,
    counter: u32,
}

impl From<Duration> for GrainInitializationDelayHandle {
    fn from(grain_initialization_delay: Duration) -> Self {
        GrainInitializationDelayHandle::new(grain_initialization_delay)
    }
}

impl GrainInitializationDelayHandle {
    pub fn get(&self) -> Duration {
        *self.grain_initialization_delay.lock().unwrap()
    }

    pub fn set(&mut self, grain_initialization_delay: impl Into<Duration>) {
        *self.grain_initialization_delay.lock().unwrap() =
            GranularSynthesizer::sanitize_grain_initialization_delay(
                grain_initialization_delay.into(),
            );
        self.bump_counter();
    }

    pub fn new(grain_initialization_delay: impl Into<Duration>) -> Self {
        GrainInitializationDelayHandle {
            grain_initialization_delay: Arc::new(Mutex::new(
                GranularSynthesizer::sanitize_grain_initialization_delay(
                    grain_initialization_delay.into(),
                ),
            )),
            counter: Default::default(),
        }
    }
}

impl BumpCounter for GrainInitializationDelayHandle {
    fn bump_counter(&mut self) {
        self.counter = self.counter.wrapping_add(1);
    }
}

impl PartialEq for GrainInitializationDelayHandle {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get() && self.counter == other.counter
    }
}

impl Default for GrainInitializationDelayHandle {
    fn default() -> Self {
        Self {
            grain_initialization_delay: Arc::new(Mutex::new(Default::default())),
            counter: Default::default(),
        }
    }
}
