use cpal::Stream;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// A wrapper around `cpal`'s Stream type for implementing `PartialEq`, etc.
#[derive(Clone, Default)]
pub struct StreamHandle {
    stream: Arc<Mutex<Option<Stream>>>,
    uuid: Uuid,
}

impl PartialEq for StreamHandle {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for StreamHandle {}

impl Debug for StreamHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamHandle")
            .field("uuid", &self.uuid)
            .finish()
    }
}

impl StreamHandle {
    pub fn new(stream: Stream) -> Self {
        StreamHandle {
            stream: Arc::new(Mutex::new(Some(stream))),
            uuid: Uuid::new_v4(),
        }
    }

    pub fn take(&self) -> Option<Stream> {
        self.stream.lock().unwrap().take()
    }
}
