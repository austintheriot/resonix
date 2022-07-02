use super::{
    bump_counter::BumpCounter, recording_status::RecordingStatus,
    recording_status_action::RecordingStatusAction,
};
use std::sync::{Arc, Mutex};

/// Current recording/stopped status -- for use in both UI and audio processing
#[derive(Clone, Debug)]
pub struct RecordingStatusHandle {
    play_status: Arc<Mutex<RecordingStatus>>,
    counter: u32,
}

impl RecordingStatusAction for RecordingStatusHandle {
    fn new(play_status: RecordingStatus) -> Self {
        RecordingStatusHandle {
            play_status: Arc::new(Mutex::new(play_status)),
            counter: Default::default(),
        }
    }

    fn get(&self) -> RecordingStatus {
        *self.play_status.lock().unwrap()
    }

    fn set(&mut self, status: RecordingStatus) {
        *self.play_status.lock().unwrap() = status;
        self.bump_counter();
    }
}

impl BumpCounter for RecordingStatusHandle {
    fn bump_counter(&mut self) {
        self.counter = self.counter.wrapping_add(1);
    }
}

impl PartialEq for RecordingStatusHandle {
    fn eq(&self, other: &Self) -> bool {
        self.counter == other.counter && self.get() == other.get()
    }
}

impl Default for RecordingStatusHandle {
    fn default() -> Self {
        Self {
            play_status: Arc::new(Mutex::new(RecordingStatus::Stop)),
            counter: Default::default(),
        }
    }
}
