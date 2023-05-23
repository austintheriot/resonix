use super::{bump_counter::BumpCounter, density::Density, density_action::DensityAction};
use audio::{
    granular_synthesizer::GranularSynthesizer,
    granular_synthesizer_action::GranularSynthesizerAction, percentage::Percentage,
};
use std::sync::{Arc, Mutex};

/// Wrapper around `Density`, which makes it possible to access
/// the data from the audio thread, while also updating the value from the UI.
#[derive(Clone, Debug)]
pub struct DensityHandle {
    density: Arc<Mutex<Density>>,
    counter: u32,
}

impl From<f32> for DensityHandle {
    fn from(density: f32) -> Self {
        DensityHandle::new(density)
    }
}

impl DensityAction for DensityHandle {
    const DEFAULT_DENSITY: f32 = GranularSynthesizer::DEFAULT_DENSITY;

    fn new(density: impl Into<Percentage>) -> Self {
        DensityHandle {
            density: Arc::new(Mutex::new(Density::new(density))),
            counter: Default::default(),
        }
    }

    fn get(&self) -> Percentage {
        self.density.lock().unwrap().get()
    }

    fn set(&mut self, density: impl Into<Percentage>) {
        self.density.lock().unwrap().set(density);
        self.bump_counter();
    }
}

impl BumpCounter for DensityHandle {
    fn bump_counter(&mut self) {
        self.counter = self.counter.wrapping_add(1);
    }
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
