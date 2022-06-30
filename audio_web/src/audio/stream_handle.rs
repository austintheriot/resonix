use cpal::Stream;
use std::fmt::Debug;
use std::sync::Arc;
use uuid::Uuid;

/// A wrapper around `cpal`'s Stream type for implementing `PartialEq`, etc.
#[derive(Clone)]
pub struct StreamHandle {
    _stream: Arc<Stream>,
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
            _stream: Arc::new(stream),
            uuid: Uuid::new_v4(),
        }
    }
}
