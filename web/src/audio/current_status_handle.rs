use super::current_status::CurrentStatus;
use std::sync::{Arc, Mutex};

/// Current play/pause status -- for use in both UI and audio processing
#[derive(Clone, Debug)]
pub struct CurrentStatusHandle {
    current_status: Arc<Mutex<CurrentStatus>>,
    counter: u32,
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

impl CurrentStatusHandle {
    /// Bumps up counter so that Yew knows interanal state has changed,
    /// even when the internal current_status points to the same memory
    fn bump_counter(&mut self) {
        self.counter = self.counter.wrapping_add(1);
    }

    pub fn new(current_status: CurrentStatus) -> Self {
        CurrentStatusHandle {
            current_status: Arc::new(Mutex::new(current_status)),
            counter: Default::default(),
        }
    }

    pub fn get(&self) -> CurrentStatus {
        *self.current_status.lock().unwrap()
    }

    pub fn set(&mut self, status: CurrentStatus) {
        *self.current_status.lock().unwrap() = status;
        self.bump_counter();
    }
}
