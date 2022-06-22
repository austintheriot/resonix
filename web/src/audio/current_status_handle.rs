use super::{current_status::CurrentStatus, current_status_action::CurrentStatusAction, bump_counter::BumpCounter};
use std::sync::{Arc, Mutex};

/// Current play/pause status -- for use in both UI and audio processing
#[derive(Clone, Debug)]
pub struct CurrentStatusHandle {
    current_status: Arc<Mutex<CurrentStatus>>,
    counter: u32,
}

impl CurrentStatusAction for CurrentStatusHandle {
    fn new(current_status: CurrentStatus) -> Self {
        CurrentStatusHandle {
            current_status: Arc::new(Mutex::new(current_status)),
            counter: Default::default(),
        }
    }

    fn get(&self) -> CurrentStatus {
        *self.current_status.lock().unwrap()
    }

    fn set(&mut self, status: CurrentStatus) {
        *self.current_status.lock().unwrap() = status;
        self.bump_counter();
    }
}

impl BumpCounter for CurrentStatusHandle {
    fn bump_counter(&mut self) {
        self.counter = self.counter.wrapping_add(1);
    }
}

impl PartialEq for CurrentStatusHandle {
    fn eq(&self, other: &Self) -> bool {
        self.counter == other.counter && self.get() == other.get()
    }
}

impl Default for CurrentStatusHandle {
    fn default() -> Self {
        Self {
            current_status: Arc::new(Mutex::new(CurrentStatus::PAUSE)),
            counter: Default::default(),
        }
    }
}