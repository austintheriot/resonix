use std::fmt::Debug;
use std::sync::Arc;
use uuid::Uuid;

/// Adds a Uuid to Buffer to make `PartialEq` diffs faster
/// since (at least currently), it is not expected to modify the buffer in place
#[derive(Clone, Default)]
pub struct Buffer {
    pub data: Arc<Vec<f32>>,
    uuid: Uuid,
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
