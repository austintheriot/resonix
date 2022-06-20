use std::sync::{Arc, Mutex};

use uuid::Uuid;

pub const GAIN_MIN: f32 = -1.0;
pub const GAIN_MAX: f32 = 1.0;

#[derive(Clone, Debug)]
pub struct Gain {
    gain: Arc<Mutex<f32>>,
    uuid: Uuid,
}

impl PartialEq for Gain {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get() && self.uuid == other.uuid
    }
}

impl Default for Gain {
    fn default() -> Self {
        Self {
            gain: Arc::new(Mutex::new(1.0)),
            uuid: Default::default(),
        }
    }
}

impl Gain {
    fn sanitize_gain(input_gain: f32) -> f32 {
        input_gain.max(GAIN_MIN).min(GAIN_MAX)
    }

    pub fn new(gain: f32) -> Self {
        Gain {
            gain: Arc::new(Mutex::new(Gain::sanitize_gain(gain))),
            uuid: Uuid::new_v4()
        }
    }

    pub fn get(&self) -> f32 {
        *self.gain.lock().unwrap()
    }

    pub fn set(&self, gain: f32) {
        *self.gain.lock().unwrap() = gain;
    }
}
