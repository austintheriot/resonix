use super::buffer_selection::BufferSelection;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// This is an `Arc` wrapper around `BufferSelection`
/// 
/// Wrapping `BufferSelection` in an `Arc` allows up-to-date state in BufferSelection to be 
/// accesed from within the separate audio thread for up-to-date audio processing. 
/// 
/// When the `BufferHandle` is `Clone`d on global state updates, it's clone is the 
/// identical object to the original, because the `buffer_selection`'s `Arc` is simply cloned.
/// 
/// This makes it impossible to know when the `buffer_selection`'s data has actually been
/// udpated. 
/// 
/// For this reason, `clone_with_new_id` is provided to "force" a difference in state
/// while keeping the underlying `buffer_selection` reference identical in memory,
/// which can be used on state updates that are assumed to change the `buffer_selection`.
#[derive(Clone, Debug, Default)]
pub struct BufferHandle {
    pub buffer_selection: Arc<Mutex<BufferSelection>>,
    uuid: Uuid,
}

impl BufferHandle {
    /// Clones the existing buffer selection, but with a new uuid.
    /// This allows state diffing to know when the buffer selection has been modified.
    pub fn clone_with_new_id(&self) -> Self {
        Self {
            buffer_selection: Arc::clone(&self.buffer_selection),
            uuid: Uuid::new_v4(),
        }
    }
}

impl BufferHandle {
    pub fn new(buffer_selection: Arc<Mutex<BufferSelection>>) -> Self {
        BufferHandle {
            buffer_selection,
            uuid: Uuid::new_v4(),
        }
    }
}

impl PartialEq for BufferHandle {
    fn eq(&self, other: &Self) -> bool {
        let self_buffer_selection = self.buffer_selection.lock().unwrap().clone();
        let other_buffer_selection = other.buffer_selection.lock().unwrap().clone();
        self.uuid == other.uuid && self_buffer_selection == other_buffer_selection
    }
}

impl Eq for BufferHandle {}
