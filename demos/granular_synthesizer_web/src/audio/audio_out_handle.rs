use resonix::{DAC, NodeUid};
use std::ops::Deref;
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

pub struct AudioOutHandle {
    data: Arc<Mutex<Option<DAC>>>,
    uid: NodeUid,
}

impl From<DAC> for AudioOutHandle {
    fn from(data: DAC) -> Self {
        Self {
            data: Arc::new(Mutex::new(Some(data))),
            uid: 0,
        }
    }
}

impl Clone for AudioOutHandle {
    fn clone(&self) -> Self {
        AudioOutHandle {
            data: Arc::clone(&self.data),
            uid: self.uid,
        }
    }
}

impl PartialEq for AudioOutHandle {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl Debug for AudioOutHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioOutHandle")
            .field("uid", &self.uid)
            .finish()
    }
}

impl Default for AudioOutHandle {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(None)),
            uid: 0,
        }
    }
}

impl Eq for AudioOutHandle {}

impl Deref for AudioOutHandle {
    type Target = Arc<Mutex<Option<DAC>>>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
