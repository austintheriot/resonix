use resonix::AudioPlayer;
use std::ops::Deref;
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

pub struct AudioPlayerHandle {
    data: Arc<Mutex<Option<AudioPlayer<()>>>>,
    uid: u32,
}

impl From<AudioPlayer<()>> for AudioPlayerHandle {
    fn from(data: AudioPlayer<()>) -> Self {
        Self {
            data: Arc::new(Mutex::new(Some(data))),
            uid: 0,
        }
    }
}

impl Clone for AudioPlayerHandle {
    fn clone(&self) -> Self {
        AudioPlayerHandle {
            data: Arc::clone(&self.data),
            uid: self.uid,
        }
    }
}

impl PartialEq for AudioPlayerHandle {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl Debug for AudioPlayerHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioPlayerHandle")
            .field("uid", &self.uid)
            .finish()
    }
}

impl Default for AudioPlayerHandle {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(None)),
            uid: 0,
        }
    }
}

impl Eq for AudioPlayerHandle {}

impl Deref for AudioPlayerHandle {
    type Target = Arc<Mutex<Option<AudioPlayer<()>>>>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
