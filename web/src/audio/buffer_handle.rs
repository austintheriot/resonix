use std::fmt::Debug;
use std::sync::Arc;
use uuid::Uuid;

/// Adds a Uuid to Buffer to make `PartialEq` diffs faster,
/// since (at least currently) it is not expected to modify the buffer in place.
///
/// It is only important that newly created buffers be distinguishable from one another.
///
/// For a different approach to this problem (with different constraints), see ```buffer_selection_handle```
#[derive(Clone, Default)]
pub struct BufferHandle {
    data: Arc<Vec<f32>>,
    uuid: Uuid,
}

impl PartialEq for BufferHandle {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for BufferHandle {}

impl Debug for BufferHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Buffer").field("uuid", &self.uuid).finish()
    }
}

impl BufferHandle {
    pub fn new(data: Arc<Vec<f32>>) -> Self {
        BufferHandle {
            data: Arc::clone(&data),
            uuid: Uuid::new_v4(),
        }
    }

    pub fn get_data(&self) -> Arc<Vec<f32>> {
        Arc::clone(&self.data)
    }
}
