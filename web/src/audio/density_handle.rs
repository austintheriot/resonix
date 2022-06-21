use std::sync::{Arc, Mutex};

use common::granular_synthesizer::GranularSynthesizer;

use super::density::Density;

/// Wrapper around `Density`, which makes it possible to access
/// the data from the audio thread, while also updating the value from the UI.
#[derive(Clone, Debug)]
pub struct DensityHandle {
    density: Arc<Mutex<Density>>,
    counter: u32,
}

impl PartialEq for DensityHandle {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get() && self.counter == other.counter
    }
}

impl Default for DensityHandle {
    fn default() -> Self {
        Self {
            density: Arc::new(Mutex::new(Density::default())),
            counter: Default::default(),
        }
    }
}

impl DensityHandle {
    /// Bumps up counter so that Yew knows interanal state has changed,
    /// even when the internal ```density``` points to the same memory
    ///
    /// (i.e. now the external handle wrapper has a different count than
    /// handle that it was cloned from, so they will no longer be ==)
    fn bump_counter(&mut self) {
        self.counter = self.counter.wrapping_add(1);
    }

    pub fn new(density: f32) -> Self {
        DensityHandle {
            density: Arc::new(Mutex::new(Density::new(
                GranularSynthesizer::sanitize_density(density),
            ))),
            counter: Default::default(),
        }
    }

    pub fn get(&self) -> f32 {
        self.density.lock().unwrap().0
    }

    pub fn set(&mut self, density: f32) {
        self.density.lock().unwrap().0 = GranularSynthesizer::sanitize_density(density);
        self.bump_counter();
    }
}
