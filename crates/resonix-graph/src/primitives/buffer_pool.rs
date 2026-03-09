use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

use alloc::boxed::Box;

use crate::{primitives::ConnectionId, utils::IntMap};

use super::Sample;

/// A pooled audio buffer with its channel count.
///
/// `data` holds `block_size * channels` samples in planar layout.
/// The slice is wrapped in `UnsafeCell` so that raw pointers derived from
/// `UnsafeCell::get()` carry SharedReadWrite (SRW) provenance under Stacked
/// Borrows, preventing invalidation when the compiled plan holds both an
/// input pointer (downstream node reads) and an output pointer (upstream
/// node writes) to the same buffer simultaneously.
#[derive(Debug)]
pub struct ChannelledBuffer {
    pub channels: usize,
    pub data: Box<UnsafeCell<[Sample]>>,
}

#[derive(Debug, Default)]
pub struct BufferPool {
    buffers: IntMap<ConnectionId, ChannelledBuffer>,
}

impl Deref for BufferPool {
    type Target = IntMap<ConnectionId, ChannelledBuffer>;

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
