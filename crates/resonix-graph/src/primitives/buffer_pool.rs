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
