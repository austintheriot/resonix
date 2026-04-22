use core::ops::{Deref, DerefMut};

use hashbrown::HashMap;

use crate::{implementations::OwnedAudioBuffer, primitives::ConnectionId};

#[derive(Debug, Default)]
pub struct BufferPool {
    // TODO: replace with a Vec for better caching/lookup speeds
    // but consider if this should be a pub struct or a pub(crate) struct:
    // `ConnectionId`s are guaranteed to be dense in THIS implementation
    buffers: HashMap<ConnectionId, OwnedAudioBuffer>,
}

impl Deref for BufferPool {
    type Target = HashMap<ConnectionId, OwnedAudioBuffer>;

    fn deref(&self) -> &Self::Target {
        &self.buffers
    }
}

impl DerefMut for BufferPool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_buffer_pool_is_empty() {
        let pool = BufferPool::default();
        assert!(pool.is_empty());
    }

    #[test]
    fn deref_of_default_pool_yields_empty_map() {
        let pool = BufferPool::default();
        assert_eq!(pool.len(), 0);
    }

    #[test]
    fn deref_mut_allows_removing_entries() {
        let mut pool = BufferPool::default();
        // Insert via the underlying map (requires a ChannelledBuffer, so we
        // only verify that nothing is present before and after a no-op removal).
        let absent_id = ConnectionId::new(999);
        let removed = pool.remove(&absent_id);
        assert!(removed.is_none());
    }
}
