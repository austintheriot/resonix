use std::sync::Arc;

use cpal::Stream;
use serde::{Serialize, ser::SerializeStruct};
use std::fmt::Debug;
use uuid::Uuid;

/// A wrapper around `cpal`'s Stream type for implementing `PartialEq`, etc.
#[derive(Clone)]
pub struct StreamHandle {
    _stream: Arc<Stream>,
    uuid: Uuid,
}

/// This is only serialized for state update logging purposes
impl Serialize for StreamHandle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
            // 3 is the number of fields in the struct.
            let mut state = serializer.serialize_struct("StreamHandle", 1)?;
            state.serialize_field("uuid", &self.uuid.to_string())?;
            state.end()
    }
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
