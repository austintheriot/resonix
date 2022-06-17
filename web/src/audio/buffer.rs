use std::fmt::Debug;
use std::sync::Arc;
use serde::{Serialize, ser::SerializeStruct};
use uuid::Uuid;

/// Adds a Uuid to Buffer to make `PartialEq` diffs faster
/// since (at least currently), it is not expected to modify the buffer in place
#[derive(Clone, Default)]
pub struct Buffer {
    pub data: Arc<Vec<f32>>,
    uuid: Uuid,
}

/// This is only serialized for state update logging purposes
impl Serialize for Buffer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
            // 3 is the number of fields in the struct.
            let mut state = serializer.serialize_struct("Buffer", 1)?;
            state.serialize_field("uuid", &self.uuid.to_string())?;
            state.end()
    }
}

impl PartialEq for Buffer {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for Buffer {}

impl Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Buffer").field("uuid", &self.uuid).finish()
    }
}

impl Buffer {
    pub fn new(data: Arc<Vec<f32>>) -> Self {
        Buffer {
            data: Arc::clone(&data),
            uuid: Uuid::new_v4(),
        }
    }
}
