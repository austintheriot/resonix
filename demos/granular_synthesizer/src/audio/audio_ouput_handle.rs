use super::{audio_output::AudioOutput, audio_output_action::AudioOutputAction};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone, Debug)]
/// Stores the most recent frame data for visualization purposes
///
/// This data never triggers UI re-renders, but is instead accessed from within
/// components for the purposes of painting graphical audio visualizations on a canvas.
pub struct AudioOutputHandle {
    data: Arc<Mutex<AudioOutput>>,
    uuid: Uuid,
}

impl AudioOutputHandle {
    pub fn reset_inner_to_default(&self) {
        *self.data.lock().unwrap() = Default::default();
    }
}

impl AudioOutputAction for AudioOutputHandle {
    fn add_frame(&mut self, frame: Vec<f32>) {
        self.data.lock().unwrap().add_frame(frame)
    }

    fn get_simple_moving_average(&self) -> Vec<f32> {
        self.data.lock().unwrap().get_simple_moving_average()
    }
}

impl Default for AudioOutputHandle {
    fn default() -> Self {
        Self {
            data: Default::default(),
            uuid: Uuid::new_v4(),
        }
    }
}

impl PartialEq for AudioOutputHandle {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for AudioOutputHandle {}
