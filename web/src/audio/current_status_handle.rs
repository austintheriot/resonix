use std::sync::{Arc, Mutex};
use uuid::Uuid;

use super::current_status::CurrentStatus;

/// Current play/pause status -- for use in both UI and audio processing
#[derive(Clone, Debug)]
pub struct CurrentStatusHandle {
    current_status: Arc<Mutex<CurrentStatus>>,
    uuid: Uuid,
}

impl PartialEq for CurrentStatusHandle {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get() && self.uuid == other.uuid
    }
}

impl Default for CurrentStatusHandle {
    fn default() -> Self {
        Self {
            current_status: Arc::new(Mutex::new(CurrentStatus::PAUSE)),
            uuid: Default::default(),
        }
    }
}

impl CurrentStatusHandle {
    pub fn new(current_status: CurrentStatus) -> Self {
        CurrentStatusHandle {
            current_status: Arc::new(Mutex::new(current_status)),
            uuid: Uuid::new_v4(),
        }
    }

    pub fn get(&self) -> CurrentStatus {
        *self.current_status.lock().unwrap()
    }

    pub fn set(&self, status: CurrentStatus) {
        *self.current_status.lock().unwrap() = status;
    }
}
