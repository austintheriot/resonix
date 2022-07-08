use super::{
    bump_counter::BumpCounter, play_status::PlayStatus, play_status_action::PlayStatusAction,
};
use std::sync::{Arc, Mutex};

/// Current play/pause status -- for use in both UI and audio processing
#[derive(Clone, Debug)]
pub struct PlayStatusHandle {
    play_status: Arc<Mutex<PlayStatus>>,
    counter: u32,
}

impl PlayStatusAction for PlayStatusHandle {
    fn new(play_status: PlayStatus) -> Self {
        PlayStatusHandle {
            play_status: Arc::new(Mutex::new(play_status)),
            counter: Default::default(),
        }
    }

    fn get(&self) -> PlayStatus {
        *self.play_status.lock().unwrap()
    }

    fn set(&mut self, status: PlayStatus) {
        *self.play_status.lock().unwrap() = status;
        self.bump_counter();
    }
}

impl BumpCounter for PlayStatusHandle {
    fn bump_counter(&mut self) {
        self.counter = self.counter.wrapping_add(1);
    }
}

impl PartialEq for PlayStatusHandle {
    fn eq(&self, other: &Self) -> bool {
        self.counter == other.counter && self.get() == other.get()
    }
}

impl Default for PlayStatusHandle {
    fn default() -> Self {
        Self {
            play_status: Arc::new(Mutex::new(PlayStatus::Pause)),
            counter: Default::default(),
        }
    }
}
